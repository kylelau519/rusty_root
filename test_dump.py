import struct
import zlib

def decompress_root_data(data):
    # Skip ROOT compression header (e.g., ZLIB or LZ4)
    # The header format is typically "CS" + method (1 byte) + c1, c2, c3 (sizes) + deflated data
    # Let's just find the zlib header (0x78)
    idx = data.find(b'\x78')
    if idx == -1:
        return data
    try:
        return zlib.decompress(data[idx:])
    except:
        return data

with open("rusty_root_io/testfiles/wzqcd_mc20a.root", "rb") as f:
    f.seek(80357582)
    key_data = f.read(64)
    compressed = f.read(8360 - 64)
    decomp = decompress_root_data(compressed)

combined = key_data + decomp
print(f"Total length: {len(combined)}")

# Now we find the first TStreamerInfo size
# TList header... let's just dump the first few bytes of combined
print("Head:", combined[:100].hex())

# Let's find "TStreamerInfo" string
idx = combined.find(b"TStreamerInfo")
print(f"Found TStreamerInfo at: {idx}")
# we know the first TStreamerInfo ends around idx + 440 ?
if idx != -1:
    print(combined[idx-10 : idx+500].hex())
