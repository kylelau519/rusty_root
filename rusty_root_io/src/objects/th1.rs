use crate::objects::tnamed::TNamed;
use crate::tree::ttree::{TAttFill, TAttLine, TAttMarker};

// TH1::Streamer (after the outer TH1F byte_count+version) calls, in this order:
// TNamed::Streamer, TAttLine::Streamer, TAttFill::Streamer, TAttMarker::Streamer.
// Note the order is Line before Fill on the wire.
#[binrw::binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TH1 {
    #[br(map = |x: u32| x & crate::core::constant::K_BYTECOUNTMASK)]
    pub byte_count: u32,
    pub version: u16,
    pub tnamed: TNamed,
    pub tatt_line: TAttLine,
    pub tatt_fill: TAttFill,
    pub tatt_marker: TAttMarker,
}

#[cfg(test)]
mod test {
    use super::TH1;
    use crate::objects::tkey::TKey;
    use crate::objects::tstring::TString;
    use binrw::BinRead;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn test_th1() {
        let path =
            "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
        let key = TKey {
            n_bytes: 612,
            version: 4,
            obj_len: 1038,
            datime: 2054579217,
            key_len: 95,
            cycle: 1,
            seek_key: 79795566,
            seek_p_dir: 100,
            class_name: TString::new("TH1F"),
            name: TString::new("cflow_AnaMuons_Baseline_NOSYS"),
            title: TString::new("Object Cutflow: AnaMuons.Baseline"),
        };
        let file = std::fs::File::open(path).expect("Failed to open ROOT file");
        let mut reader = std::io::BufReader::new(file);
        let mut decompressed_data = key
            .decompress_full(&mut reader)
            .expect("Failed to decompress TH1F data");

        // Skip the key header (key_len) and the outer TH1F byte_count+version (6 bytes)
        // to land on TH1's own byte_count+version.
        decompressed_data
            .seek(SeekFrom::Start(key.key_len as u64 + 6))
            .expect("seek failed");

        let th1 = TH1::read_be(&mut decompressed_data).expect("Failed to parse TH1");
        dbg!(&th1);

        assert_eq!(th1.tnamed.name, "cflow_AnaMuons_Baseline_NOSYS");
        assert_eq!(th1.tnamed.title, "Object Cutflow: AnaMuons.Baseline");
        assert_eq!(th1.tatt_line.f_line_color, 602);
        assert_eq!(th1.tatt_fill.f_fill_color, 0);
        assert_eq!(th1.tatt_fill.f_fill_style, 1001);
        assert_eq!(th1.tatt_marker.f_marker_color, 1);
        assert_eq!(th1.tatt_marker.f_marker_size, 1.0);
    }
}
