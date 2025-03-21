use super::cg_block::Cgblock;
use super::cn_block::Cnblock;
use super::fh_block::Fhblock;
use super::tx_block::Txblock;
use crate::mdf::{self, MDFFile, MdfChannel, RasterType, SourceInformation};
use crate::record::Record;
use crate::signal::{self, Signal};
use crate::utils;
use std::fs::File;
use std::io::prelude::*;

use super::block::{Block, LinkedBlock};
use super::dg_block::Dgblock;
use super::hd_block::Hdblock;
use super::id_block::Idblock;
use super::mdf4_enums::ChannelType;
use super::si_block::Siblock;

pub fn link_extract(
    stream: &[u8],
    position: usize,
    little_endian: bool,
    no_links: u64,
) -> (usize, Vec<u64>) {
    let mut links = Vec::new();
    let mut pos = position;

    for _i in 0..no_links {
        let address: u64 = utils::read(stream, little_endian, &mut pos);
        links.push(address);
    }

    (pos, links)
}

#[derive(Debug, Clone, PartialEq)]
pub struct MDF4 {
    id: Idblock,
    header: Hdblock,
    comment: String,
    data_groups: Vec<Dgblock>,
    channels: Vec<Cnblock>,
    channel_groups: Vec<Cgblock>,
    little_endian: bool,
    file: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ChannelGroupMetadata {
    pub source_type: String,
    pub source_name: String,
    pub source_path: String,
    pub bus_type: String,
}

#[derive(Debug, Clone)]
pub struct ChannelMetadata {
    pub description: String,
    pub unit: String,
    pub source: String,
}

impl MDF4 {
    fn read_metadata(
        &mut self,
        stream: &[u8],
        position: usize,
        little_endian: bool,
    ) -> (usize, String) {
        let mut pos = position;
        let mut comment = String::new();

        // Try to read the next block header
        if pos + 4 > stream.len() {
            return (pos, comment);
        }

        let header_bytes = &stream[pos..pos + 4];

        match header_bytes {
            b"##FH" => {
                // v4.11 style - FH block followed by TX block
                let (new_pos, fh_block) = Fhblock::read(stream, pos, little_endian);
                pos = new_pos;

                // Read the TX block that follows
                if let Some(tx_pos) = fh_block.comment_addr() {
                    if let Ok((_pos, tx_block)) =
                        Txblock::try_read(stream, tx_pos as usize, little_endian)
                    {
                        comment = tx_block.text();
                    }
                }
            }
            b"##MD" => {
                // v4.10 style - direct MD block
                let (new_pos, md_block) =
                    super::md_block::Mdblock::read(stream, pos, little_endian);
                pos = new_pos;
                comment = md_block.text();
            }
            b"##FR" => {
                // Skip FR block and continue
                let (new_pos, _) =
                    super::block_header::BlockHeader::read(stream, pos, little_endian);
                pos = new_pos;
            }
            _ => {
                // Unknown block type, skip metadata
                println!(
                    "Warning: Unknown metadata block type: {:?}",
                    String::from_utf8_lossy(header_bytes)
                );
                // Skip the block by reading its header to get the length
                let (new_pos, header) =
                    super::block_header::BlockHeader::read(stream, pos, little_endian);
                pos = new_pos + (header.length as usize - header.byte_len());
            }
        }

        (pos, comment)
    }

    pub fn get_source_information(
        &self,
        data_group: usize,
        channel_group: usize,
    ) -> Option<SourceInformation> {
        let dg = &self.data_groups[data_group];
        let channel_groups = dg
            .first(&self.file, self.little_endian)
            .list(&self.file, self.little_endian);

        let cg = &channel_groups[channel_group];

        // Get SI block from channel group
        if cg.cg_si_acq_source == 0 {
            return None;
        }

        let (_, si_block) =
            Siblock::read(&self.file, cg.cg_si_acq_source as usize, self.little_endian);

        // Get source name and path from TX blocks
        let source_name = if si_block.si_tx_name != 0 {
            let (_, tx_block) =
                Txblock::read(&self.file, si_block.si_tx_name as usize, self.little_endian);
            tx_block.text()
        } else {
            String::new()
        };

        let source_path = if si_block.si_tx_path != 0 {
            let (_, tx_block) =
                Txblock::read(&self.file, si_block.si_tx_path as usize, self.little_endian);
            tx_block.text()
        } else {
            String::new()
        };

        Some(SourceInformation {
            source_type: si_block.si_type,
            bus_type: si_block.si_bus_type,
            source_name,
            source_path,
        })
    }

    pub fn get_channel_group_metadata(
        &self,
        data_group: usize,
        channel_group: usize,
    ) -> Option<ChannelGroupMetadata> {
        let dg = &self.data_groups[data_group];
        let channel_groups = dg
            .first(&self.file, self.little_endian)
            .list(&self.file, self.little_endian);

        let cg = &channel_groups[channel_group];

        // Get SI block from channel group
        if cg.cg_si_acq_source == 0 {
            return None;
        }

        let (_, si_block) =
            Siblock::read(&self.file, cg.cg_si_acq_source as usize, self.little_endian);

        // Get source name and path from TX blocks
        let source_name = if si_block.si_tx_name != 0 {
            let (_, tx_block) =
                Txblock::read(&self.file, si_block.si_tx_name as usize, self.little_endian);
            tx_block.text()
        } else {
            String::new()
        };

        let source_path = if si_block.si_tx_path != 0 {
            let (_, tx_block) =
                Txblock::read(&self.file, si_block.si_tx_path as usize, self.little_endian);
            tx_block.text()
        } else {
            String::new()
        };

        Some(ChannelGroupMetadata {
            source_type: format!("{:?}", si_block.si_type),
            source_name,
            source_path,
            bus_type: format!("{:?}", si_block.si_bus_type),
        })
    }

    pub fn get_channel_metadata(
        &self,
        data_group: usize,
        channel_group: usize,
        channel: usize,
    ) -> Option<ChannelMetadata> {
        let dg = &self.data_groups[data_group];
        let channel_groups = dg
            .first(&self.file, self.little_endian)
            .list(&self.file, self.little_endian);

        let cg = &channel_groups[channel_group];
        let channels = cg
            .first(&self.file, self.little_endian)
            .list(&self.file, self.little_endian);

        let cn = &channels[channel];

        // Get metadata from TX blocks
        let description = if cn.cn_md_comment != 0 {
            let (_, tx_block) =
                Txblock::read(&self.file, cn.cn_md_comment as usize, self.little_endian);
            tx_block.text()
        } else {
            String::new()
        };

        let unit = if cn.cn_md_unit != 0 {
            let (_, tx_block) =
                Txblock::read(&self.file, cn.cn_md_unit as usize, self.little_endian);
            tx_block.text()
        } else {
            String::new()
        };

        let source = if cn.cn_si_source != 0 {
            let (_, si_block) =
                Siblock::read(&self.file, cn.cn_si_source as usize, self.little_endian);
            if si_block.si_tx_name != 0 {
                let (_, tx_block) =
                    Txblock::read(&self.file, si_block.si_tx_name as usize, self.little_endian);
                tx_block.text()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        Some(ChannelMetadata {
            description,
            unit,
            source,
        })
    }
}

impl MDFFile for MDF4 {
    fn channels(&self) -> Vec<MdfChannel> {
        let mut mdf_channels = Vec::new();

        let little_endian = true;

        let (position, _id_block) = Idblock::read(&self.file, 0, little_endian);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);

        let next_dg = hd_block.first_data_group(&self.file, little_endian);

        let data_groups = next_dg.list(&self.file, little_endian);

        for (dg_no, dg) in data_groups.iter().enumerate() {
            let first_cg = dg.first(&self.file, little_endian);
            let channel_groups = first_cg.list(&self.file, little_endian);

            for (cg_no, cg) in channel_groups.iter().enumerate() {
                let first_cn = cg.first(&self.file, little_endian);
                let channels = first_cn.list(&self.file, little_endian);

                for (cn_no, cn) in channels.iter().enumerate() {
                    let name = cn.clone().comment(&self.file, little_endian);
                    mdf_channels.push(mdf::MdfChannel {
                        name,
                        data_group: dg_no,
                        channel_group: cg_no,
                        channel: cn_no,
                    })
                }
            }
        }

        mdf_channels
    }

    fn find_time_channel(
        &self,
        _datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str> {
        let channel_group = self.channel_groups[channel_grp]
            .clone()
            .channels(&self.file, self.little_endian);
        for (i, channel) in channel_group.iter().enumerate() {
            if matches!(channel.channel_type(), ChannelType::Master) {
                return Ok(i);
            }
        }

        Err("No time series found for the channel group selected")
    }

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record> {
        let dg = &self.data_groups[datagroup];
        let channel_groups = dg
            .first(&self.file, self.little_endian)
            .list(&self.file, self.little_endian);

        let channel_group = &channel_groups[channel_grp];
        let channels = channel_group
            .first(&self.file, self.little_endian)
            .list(&self.file, self.little_endian);
        let cn = &channels[channel];

        let data = dg.read_data(&self.file, self.little_endian);

        let mut data_blocks: Vec<&[u8]> = vec![&[0_u8]; channel_group.record_number()];

        for (i, db) in data_blocks.iter_mut().enumerate() {
            *db = &data[(i * channel_group.record_size())..((i + 1) * channel_group.record_size())];
        }

        let byte_offset = cn.byte_offset();

        let mut records = Vec::new();
        let mut pos = 0;
        for _i in 0..channel_group.record_number() {
            records.push(&data[pos..pos + channel_group.record_size()]);
            pos += channel_group.record_size();
        }

        let mut raw_data = Vec::new();
        let end = byte_offset + cn.data_type_len();

        for rec in records {
            raw_data.push(&rec[byte_offset..end]);
        }

        let mut extracted_data = Vec::new();
        for raw in raw_data {
            extracted_data.push(Record::new(raw, cn.data_type().copy_to_data_type_read()));
        }

        extracted_data
    }

    #[must_use]
    fn new(filepath: &str) -> Self {
        let mut file = File::open(filepath).expect("Could not read file");
        let mut stream = Vec::new();
        let _ = file.read_to_end(&mut stream);

        let little_endian = true;
        let mut position = 0;

        let (pos, id) = Idblock::read(&stream, position, little_endian);
        position = pos;

        let (pos, header) = Hdblock::read(&stream, position, little_endian);
        position = pos;

        let mut mdf = Self {
            id: id.clone(),
            header: header.clone(),
            comment: String::new(),
            data_groups: Vec::new(),
            channels: Vec::new(),
            channel_groups: Vec::new(),
            little_endian,
            file: stream.clone(),
        };

        // Read metadata based on file version
        let (_, comment) = mdf.read_metadata(&stream, position, little_endian);

        mdf.comment = comment;

        // Only load the data groups initially, defer channel loading
        mdf.data_groups = header
            .first_data_group(&stream, little_endian)
            .list(&stream, little_endian);

        mdf
    }

    fn read_all(&mut self) {
        // Load channel groups and channels only when explicitly requested
        let mut channel_groups = Vec::with_capacity(self.data_groups.len());
        let mut channels = Vec::new();

        for group in &self.data_groups {
            let mut current_cg = group.first(&self.file, self.little_endian);

            loop {
                let first_cn = current_cg.first(&self.file, self.little_endian);
                let mut current_cn = first_cn;

                // Add current channel group
                channel_groups.push(current_cg.clone());

                // Collect all channels in this group
                loop {
                    channels.push(current_cn.clone());

                    if let Some(next_cn) = current_cn.next(&self.file, self.little_endian) {
                        current_cn = next_cn;
                    } else {
                        break;
                    }
                }

                // Move to next channel group if it exists
                if let Some(next_cg) = current_cg.next(&self.file, self.little_endian) {
                    current_cg = next_cg;
                } else {
                    break;
                }
            }
        }

        self.channel_groups = channel_groups;
        self.channels = channels;
    }

    fn list_data_groups(&mut self) {
        let little_endian = true;
        let position = 0;

        let (position, _id_block) = Idblock::read(&self.file, position, little_endian);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);

        let dg = hd_block
            .first_data_group(&self.file, little_endian)
            .list(&self.file, little_endian);
        self.data_groups = dg;
    }

    fn list_channels(&self) {
        let little_endian = true;

        let (position, _id_block) = Idblock::read(&self.file, 0, little_endian);
        let (_pos, hd_block) = Hdblock::read(&self.file, position, little_endian);

        let next_dg = hd_block.first_data_group(&self.file, little_endian);

        let data_groups = next_dg.list(&self.file, little_endian);

        for dg in data_groups {
            let first_cg = dg.first(&self.file, little_endian);
            let channel_groups = first_cg.list(&self.file, little_endian);

            for cg in channel_groups {
                let first_cn = cg.first(&self.file, little_endian);
                let channels = first_cn.list(&self.file, little_endian);

                let _ = cg.comment(&self.file, little_endian);

                for cn in channels {
                    let _ = cn.comment(&self.file, little_endian);
                }
            }
        }
    }

    #[must_use]
    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal {
        let time_channel = self.find_time_channel(datagroup, channel_grp);
        let time_channel = match time_channel {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        let time = self.read_channel(datagroup, channel_grp, time_channel);
        let some = self.read_channel(datagroup, channel_grp, channel);

        signal::Signal::new(
            time.iter().map(|x| x.extract()).collect(),
            some,
            "Unit".to_string(),
            "Measurement".to_string(),
            "This is some measurement".to_string(),
            false,
        )
    }

    fn cut(&self, _start: f64, _end: f64, _include_ends: bool, _time_from_zero: bool) {
        // let _delta = if time_from_zero { start } else { 0.0 };
    }

    fn export(&self, _format: &str, _filename: &str) {}
    fn filter(&self, _channels: &str) {}
    #[must_use]
    fn resample(&self, _raster: RasterType, _version: &str, _time_from_zero: bool) -> Self {
        self.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Rdblock {}
