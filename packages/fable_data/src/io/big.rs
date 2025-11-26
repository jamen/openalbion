use crate::{
    BigAssetInfo, BigAssetInfoHeader, BigAssetInfoHeaderSection, BigAssetInfoOwned,
    BigAssetInfoSection, BigBankInfo, BigBankInfoCount, BigBankInfoOwned, BigBankInfoSection,
    BigHeader, BigHeaderSection, TakeError,
};
use derive_more::{Display, Error};
use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
};

pub struct BigFile {
    file: BufReader<File>,
    header: BigHeader,
}

impl BigFile {
    pub fn new(mut file: File) -> Result<Self, BigFileError> {
        use BigFileError as E;

        let mut header_bytes = [0; BigHeader::BYTE_SIZE];

        file.seek(SeekFrom::Start(0)).map_err(E::SeekHeader)?;

        file.read_exact(&mut header_bytes).map_err(E::ReadHeader)?;

        let header = BigHeader::parse(&mut &header_bytes[..]).map_err(E::ParseHeader)?;

        let file = BufReader::new(file);

        Ok(Self { file, header })
    }

    pub fn read_bank_infos(&mut self) -> Result<Vec<BigBankInfoOwned>, BigFileError> {
        use BigFileError as E;

        let mut bank_info_count_bytes = [0u8; BigBankInfoCount::BYTE_SIZE];

        self.file
            .seek(SeekFrom::Start(self.header.banks_position as u64))
            .map_err(E::SeekBankInfoCount)?;

        self.file
            .read_exact(&mut bank_info_count_bytes)
            .map_err(E::ReadBankInfoCount)?;

        let BigBankInfoCount { bank_info_count } =
            BigBankInfoCount::parse(&mut &bank_info_count_bytes[..])
                .map_err(E::ParseBankInfoCount)?;

        let mut bank_info_bytes = Vec::new();

        self.file
            .read_to_end(&mut bank_info_bytes)
            .map_err(E::ReadBankInfoBytes)?;

        let mut bank_info_input = &bank_info_bytes[..];

        let mut bank_infos = Vec::new();

        for i in 0..bank_info_count {
            let bank_info =
                BigBankInfo::parse(&mut bank_info_input).map_err(|e| E::ParseBankInfo(i, e))?;
            bank_infos.push(bank_info.into_owned());
        }

        Ok(bank_infos)
    }

    pub fn read_asset_infos(
        &mut self,
        bank_info: &BigBankInfo,
    ) -> Result<(BigAssetInfoHeader, Vec<BigAssetInfoOwned>), BigFileError> {
        use BigFileError as E;

        let mut asset_info_bytes = vec![0u8; bank_info.length as usize];

        self.file
            .seek(SeekFrom::Start(bank_info.position as u64))
            .map_err(E::SeekAssetInfoBytes)?;

        self.file
            .read_exact(&mut asset_info_bytes)
            .map_err(E::ReadAssetInfoBytes)?;

        let mut asset_infos_input = &asset_info_bytes[..];

        let asset_info_header =
            BigAssetInfoHeader::parse(&mut asset_infos_input).map_err(E::ParseAssetInfoHeader)?;

        let mut asset_infos = Vec::with_capacity(bank_info.asset_count as usize);

        for i in 0..bank_info.asset_count {
            let asset_info =
                BigAssetInfo::parse(&mut asset_infos_input).map_err(|e| E::ParseAssetInfo(i, e))?;
            asset_infos.push(asset_info.into_owned());
        }

        Ok((asset_info_header, asset_infos))
    }

    pub fn read_asset_bytes(&mut self, asset_info: &BigAssetInfo) -> Result<Vec<u8>, BigFileError> {
        use BigFileError as E;

        let mut asset_bytes = vec![0u8; asset_info.size as usize];

        self.file
            .seek(SeekFrom::Start(asset_info.start as u64))
            .map_err(E::SeekAssetBytes)?;

        self.file
            .read_exact(&mut asset_bytes)
            .map_err(E::ReadAssetBytes)?;

        Ok(asset_bytes)
    }
}

#[derive(Debug, Display)]
pub enum BigFileError {
    SeekHeader(io::Error),
    ReadHeader(io::Error),
    ParseHeader(BigHeaderSection),
    ReadBankInfoBytes(io::Error),
    SeekBankInfoCount(io::Error),
    ReadBankInfoCount(io::Error),
    ParseBankInfoCount(TakeError),
    #[display("ParseBankInfo(${_0}, ${_1})")]
    ParseBankInfo(u32, BigBankInfoSection),
    SeekAssetInfoBytes(io::Error),
    ReadAssetInfoBytes(io::Error),
    ParseAssetInfoHeader(BigAssetInfoHeaderSection),
    #[display("ParseAssetInfo(${_0}, ${_1})")]
    ParseAssetInfo(u32, BigAssetInfoSection),
    SeekAssetBytes(io::Error),
    ReadAssetBytes(io::Error),
}
