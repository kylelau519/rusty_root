#[cfg(test)]
mod test {
    use crate::core::utils::debug_in_ascii;
    use crate::objects::tkey::TKey;
    use crate::objects::tstring::TString;
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
        let decompressed_data = key
            .decompress_full(&mut reader)
            .expect("Failed to decompress TH1F data");
        dbg!(debug_in_ascii(&decompressed_data.get_ref()[95..]));
    }
}
