use binrw::BinRead;

#[binrw::binread]
#[br(big)]
#[derive(Debug, Default)]
pub struct TArray<T>
where
    T: BinRead + 'static,
    for<'a> T: BinRead<Args<'a> = ()>,
{
    n: u32,
    #[br(count = n)]
    pub data: Vec<T>,
}
