# Offline reference set — dynamic streamer interpreter

Downloaded 2026-07-25 for offline work (no wifi). Covers the exact gap in
`rusty_root_io/src/streamer/registry.rs` (`build_field_reader`): the `todo!()`
branches for `VariableArray`, `Base`, `Object`, `StlVector`, `Unknown`, and the
"class already seen" back-reference case in `ObjectPointer`.

Everything here is a plain-text snapshot (source `master`/`main` as of the
download date) — grep them freely, no internet needed.

## 1. `root-format-spec/` — the authoritative byte-level format

From `root-project/root`'s `io/doc/TFile/`. This is ROOT's own maintained spec
of the on-disk layout, in prose+diagram form — read this first for *what the
bytes mean*, before diving into the C++.

- `streamerinfo.md` — layout of the StreamerInfo TList/TStreamerInfo record itself
- `tfile.md`, `header.md` — file header, magic, seek offsets
- `dobject.md`, `tobject.md` — the generic "any ROOT object" framing (byte_count + version + fields) — this is what your `ObjectPointer` reader already partially reimplements
- `keyslist.md`, `tdirectory.md` — TKey / TDirectory layout
- `ttree.md`, `tclonesarray.md` — TTree/TBranch/TClonesArray specifics (for later, once trees come up)
- `tprocessid.md`, `tref.md`, `trefarray.md` — object references/UIDs (relevant to the back-reference TODO — see below)
- `freesegments.md`, `gap.md` — free space records (not needed for reading)
- `DataModelEvolution.txt` — schema evolution: how ROOT reconciles an on-disk
  class version against the currently-loaded one. Relevant background for why
  `TStreamerInfo::fClassVersion`/checksum exist at all.
- `index.md` — io/io module overview

## 2. `root-source/` — the real C++ implementation

The ground truth. Your `ClassSchema` / `FieldKind` / `CompiledReader` design in
`registry.rs` is a Rust reinterpretation of ROOT's "compiled streamer action
list" — literally the same idea as `TStreamerInfo::Compile()` building a
`TStreamerInfoActions::TActionSequence` of function pointers. Read that
function first; everything else hangs off it.

| File | What to look at | Maps to your TODO |
|---|---|---|
| `TStreamerInfoActions.cxx` | `TStreamerInfo::Compile()` (~line 4133) — the switch over element kinds that appends one action per field, exactly what `build_field_reader`'s `match field_kind` does | all of them |
| `TStreamerInfoActions.cxx` | `GenericReadAction` (~line 250) | `FieldKind::Object` / `Base` — reads a nested object inline (no pointer tag), just recurses into the sub-schema |
| `TStreamerInfoActions.cxx` | `ReadSTL` (~line 1157) and the `ReadSTLMemberWise*`/`ReadSTLObjectWise*` family (~line 772–1046) | `FieldKind::StlVector` — note ROOT has both "object-wise" (bulk) and "member-wise" (per-field, used across schema versions) STL reading modes |
| `TBufferFile.cxx` | `kNewClassTag`/`kClassMask`/`kByteCountMask` consts (~line 50) and `ReadClass()` (~line 2751 onward, the tag/back-reference decode) | `FieldKind::ObjectPointer`'s `panic!` case — this is the actual algorithm for resolving "class already seen earlier in this buffer": TBufferFile keeps a `fClassMap`/`fObjectMap` (index → class or object) built up as it reads, and a back-reference tag is an index into that map, not a byte count |
| `TStreamerElement.h/.cxx` | `TStreamerBasicPointer`, `GetArrayLength()`/`GetArrayDim()` | `FieldKind::VariableArray` — the count is read from a *sibling member* named by the streamer element, resolved by name at Compile() time, not re-read from the stream |
| `TVirtualStreamerInfo.h` | `EReadWrite` enum | cross-check against your `f_type_to_field_kind` table in `tstreamer_element.rs` — should match 1:1 |
| `ESTLType.h` | STL container type codes (vector/map/set/...) | cross-check against `TStreamerType::TStreamerSTL { stl_type, c_type }` |
| `TStreamerInfoReadBuffer.cxx` | member-wise STL reading for the "changed class" schema-evolution path | secondary, only needed once you hit real schema-evolution STL fields |

C++ is verbose — don't try to read these top to bottom. Grep for the function
names above and read just those bodies; the surrounding template machinery
(`INLINE_TEMPLATE_ARGS`, `TConfiguration` boilerplate) is C++-specific
plumbing you don't need to port.

## 3. `uproot5-source/` — a from-scratch reimplementation, much easier to read

`scikit-hep/uproot5` is Python, does the exact same job as this crate (read
ROOT files without the ROOT C++ dictionary), and its source doubles as very
readable documentation. When the C++ is too dense, check here first.

| File | What's in it |
|---|---|
| `streamers.py` | Parses `TStreamerInfo`/`TStreamerElement` records — `Model_TStreamerBasicPointer` (~L779, variable arrays), `Model_TStreamerSTL`/`STLstring` (~L1273), `Model_TStreamerObjectPointer` (~L1532). Directly comparable to your `tstreamer_element.rs`. |
| `containers.py` | `AsVector`/`AsVectorLike` (~L1086/851) build the actual runtime reader for an STL vector field: read a 4-byte count, then that many elements. `STLVector`/`STLMap`/`STLSet` (~L1620+) are the runtime container types. This is the cleanest reference for your `StlVector` TODO. |
| `deserialization.py` | Generic "read one object" dispatch — the pointer/tag/back-reference logic, Python-side equivalent of `TBufferFile::ReadClass`. |
| `model.py` | Base class every generated model inherits — shows how a class's fields get read in schema order, base classes first, same ordering constraint your `ClassSchema.fields` comment calls out. |
| `reading.py` | Top-level file/key opening — background only, you've already got this working. |

## Suggested reading order for the flight

1. `root-format-spec/streamerinfo.md` and `dobject.md` — refresh on the byte framing you're already relying on.
2. `uproot5-source/containers.py` (`AsVector`, ~L1086) — simplest possible STL vector reader, port the *shape* of this to `FieldKind::StlVector`.
3. `root-source/TStreamerInfoActions.cxx`, `TStreamerInfo::Compile()` (~L4133) — skim the switch statement to see how `Base`/`Object`/`VariableArray` cases get compiled; this validates the design of `build_field_reader` itself.
4. `root-source/TBufferFile.cxx`, `ReadClass()` (~L2751) — for the back-reference `panic!`: note it needs a `fClassMap`-equivalent (`Vec<...>` or `HashMap<u32, Arc<ClassSchema>>`) threaded through the whole read, keyed by encounter order — this is a bigger structural change (needs a "reader session" object, not just a closure), so it's reasonable to scope it as a separate follow-up rather than bolting it onto `CompiledReader`'s current `Fn(&mut dyn ReadSeek)` signature.
5. `root-format-spec/tprocessid.md`/`tref.md` when you get to the back-reference case — ROOT's `TRef`/process-ID scheme is the same map-of-seen-objects idea at a higher level.

## Regenerating / updating this later

All of this was pulled from `raw.githubusercontent.com` (no auth needed) at:

```
root-project/root @ master   — io/doc/TFile/*, io/io/{inc,src}/*, core/meta/{inc,src}/TStreamerElement.*, core/meta/inc/TVirtualStreamerInfo.h, core/foundation/inc/ESTLType.h
scikit-hep/uproot5 @ main    — src/uproot/{streamers,deserialization,model,containers,reading}.py
```

Note: in ROOT's tree, the streamer-info runtime files (`TStreamerInfo*`,
`TBufferFile*`) live under `io/io/`, not `core/meta/` — only
`TStreamerElement.*` and `TVirtualStreamerInfo.h` are still under `core/meta/`.
Worth re-checking paths if you re-pull after a ROOT restructure.
