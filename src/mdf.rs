use std::fs::File;
use std::io::Read;

use crate::mdf3::mdf3_file::MDF3;
use crate::mdf4::mdf4_enums::{BusType, SourceType};
use crate::mdf4::mdf4_file::MDF4;
use crate::mdf4::mdf4_file::{ChannelGroupMetadata, ChannelMetadata};
use crate::record::Record;
use crate::signal::Signal;
use crate::utils;

#[derive(PartialEq, Debug)]
enum MDFVersion {
    MDF3,
    MDF4,
}

enum MDFType {
    MDF3(MDF3),
    MDF4(MDF4),
}

impl MDFType {
    fn check_version(filepath: &str) -> MDFVersion {
        let mut file = File::open(filepath).expect("Could not read file");
        let mut id_stream = [0_u8; 128];
        file.read_exact(&mut id_stream).unwrap();

        let mut pos = 0;
        let little_endian = true;

        let id_file: [u8; 8] = utils::read(&id_stream, little_endian, &mut pos);
        let id_vers: [u8; 8] = utils::read(&id_stream, little_endian, &mut pos);
        let _id_prog: [u8; 8] = utils::read(&id_stream, little_endian, &mut pos);
        let _id_reserved1: [u8; 4] = utils::read(&id_stream, little_endian, &mut pos);
        let _id_ver: u16 = utils::read(&id_stream, little_endian, &mut pos);
        let _id_reserved2: [u8; 34] = utils::read(&id_stream, little_endian, &mut pos);

        if !utils::eq(&id_file, &[b'M', b'D', b'F', b' ', b' ', b' ', b' ', b' ']) {
            panic!("Error: Unknown file type");
        }

        let s = String::from_utf8_lossy(&id_vers).into_owned();
        let mut version = s.split('.');
        let major_version = version.next().unwrap().parse::<usize>().unwrap();
        //let _minor_version = version.next().unwrap().parse::<usize>().unwrap();

        match major_version {
            3 => MDFVersion::MDF3,
            4 => MDFVersion::MDF4,
            _ => panic!("Unknown MDF file version"),
        }
    }

    pub fn get_channel_group_metadata(
        &self,
        data_group: usize,
        channel_group: usize,
    ) -> Option<ChannelGroupMetadata> {
        match self {
            Self::MDF3(_) => None, // MDF3 doesn't have SI blocks
            Self::MDF4(file) => file.get_channel_group_metadata(data_group, channel_group),
        }
    }

    pub fn get_channel_metadata(
        &self,
        data_group: usize,
        channel_group: usize,
        channel: usize,
    ) -> Option<ChannelMetadata> {
        match self {
            Self::MDF3(_) => None, // MDF3 doesn't have SI blocks
            Self::MDF4(file) => file.get_channel_metadata(data_group, channel_group, channel),
        }
    }
}

impl MDFFile for MDFType {
    fn channels(&self) -> Vec<MdfChannel> {
        match self {
            Self::MDF3(file) => file.channels(),
            Self::MDF4(file) => file.channels(),
        }
        // chan
    }
    fn find_time_channel(
        &self,
        datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str> {
        match self {
            Self::MDF3(file) => file.find_time_channel(datagroup, channel_grp),
            Self::MDF4(file) => file.find_time_channel(datagroup, channel_grp),
        }
    }

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record> {
        match self {
            Self::MDF3(file) => file.read_channel(datagroup, channel_grp, channel),
            Self::MDF4(file) => file.read_channel(datagroup, channel_grp, channel),
        }
    }

    #[must_use]
    fn new(filepath: &str) -> Self {
        let version = MDFType::check_version(filepath);

        match version {
            MDFVersion::MDF3 => MDFType::MDF3(MDF3::new(filepath)),
            MDFVersion::MDF4 => MDFType::MDF4(MDF4::new(filepath)),
        }
    }

    fn read_all(&mut self) {
        match self {
            Self::MDF3(file) => file.read_all(),
            Self::MDF4(file) => file.read_all(),
        }
    }

    fn list_data_groups(&mut self) {
        match self {
            Self::MDF3(file) => file.list_data_groups(),
            Self::MDF4(file) => file.list_data_groups(),
        }
    }

    fn list_channels(&self) {
        match self {
            Self::MDF3(file) => file.list_channels(),
            Self::MDF4(file) => file.list_channels(),
        }
    }

    #[must_use]
    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal {
        match self {
            Self::MDF3(file) => file.read(datagroup, channel_grp, channel),
            Self::MDF4(file) => file.read(datagroup, channel_grp, channel),
        }
    }

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool) {
        match self {
            Self::MDF3(file) => file.cut(start, end, include_ends, time_from_zero),
            Self::MDF4(file) => file.cut(start, end, include_ends, time_from_zero),
        }
    }

    fn export(&self, format: &str, filename: &str) {
        match self {
            Self::MDF3(file) => file.export(format, filename),
            Self::MDF4(file) => file.export(format, filename),
        }
    }

    fn filter(&self, channels: &str) {
        match self {
            Self::MDF3(file) => file.filter(channels),
            Self::MDF4(file) => file.filter(channels),
        }
    }

    #[must_use]
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self {
        match self {
            Self::MDF3(file) => Self::MDF3(file.resample(raster, version, time_from_zero)),
            Self::MDF4(file) => Self::MDF4(file.resample(raster, version, time_from_zero)),
        }
    }
    // #[must_use]
    // fn select(
    //     &self,
    //     channels: ChannelsType,
    //     record_offset: isize,
    //     raw: bool,
    //     copy_master: bool,
    //     ignore_value2text_conversions: bool,
    //     record_count: isize,
    //     validate: bool,
    // ) -> Vec<Signal>;
}

pub struct MDF {
    pub filepath: String,
    file: MDFType,
    pub channels: Vec<MdfChannel>,
}

impl MDF {
    pub fn search_channels(&self, channel_name: &str) -> Vec<MdfChannel> {
        let mut results = Vec::new();
        for channel in self.channels() {
            if channel.name.contains(channel_name) {
                results.push(channel);
            }
        }
        results
    }

    pub fn search_channel_exact(&self, name: &str, dg: usize, cg: usize) -> Option<MdfChannel> {
        self.channels
            .iter()
            .find(|ch| ch.name.eq(name) && ch.data_group == dg && ch.channel_group == cg)
            .cloned()
    }

    pub fn search_channel_first(&self, channel_name: &str) -> Result<MdfChannel, &'static str> {
        let matches = self.search_channels(channel_name);
        match matches.len() {
            0 => Err("Channel not found"),
            _ => Ok(matches[0].clone()),
        }
    }

    pub fn list_channels(&self) {
        for channel in &self.channels {
            println!(
                "Channel: {}, DG: {}, CG: {}, CN: {}",
                channel.name, channel.data_group, channel.channel_group, channel.channel
            );
        }
    }

    pub fn read_channel(&self, channel: &MdfChannel) -> Signal {
        // Check if the channel exists and has data
        let signal = self.file.read(
            channel.data_group as usize,
            channel.channel_group as usize,
            channel.channel as usize,
        );
        
        // If the signal has no data, return an empty signal
        if signal.is_empty() {
            Signal {
                name: channel.name.clone(),
                comment: String::new(),
                unit: String::new(),
                samples: Vec::new(),
                timestamps: Vec::new(),
                raw: false,
            }
        } else {
            signal
        }
    }

    pub fn get_source_information(
        &self,
        data_group: usize,
        channel_group: usize,
    ) -> Option<SourceInformation> {
        match &self.file {
            MDFType::MDF3(_) => None, // MDF3 doesn't have SI blocks
            MDFType::MDF4(file) => file.get_source_information(data_group, channel_group),
        }
    }
}

impl MDFFile for MDF {
    fn channels(&self) -> Vec<MdfChannel> {
        self.file.channels()
    }

    fn find_time_channel(
        &self,
        datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str> {
        self.file.find_time_channel(datagroup, channel_grp)
    }

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record> {
        self.file.read_channel(datagroup, channel_grp, channel)
    }

    fn new(filepath: &str) -> Self {
        let file = MDFType::new(filepath);
        Self {
            filepath: filepath.to_string(),
            channels: file.channels(),
            file,
        }
    }

    fn read_all(&mut self) {
        self.file.read_all();
    }

    fn list_data_groups(&mut self) {
        self.file.list_data_groups();
    }

    fn list_channels(&self) {
        self.file.list_channels();
    }

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal {
        self.file.read(datagroup, channel_grp, channel)
    }

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool) {
        self.file.cut(start, end, include_ends, time_from_zero)
    }

    fn export(&self, format: &str, filename: &str) {
        self.file.export(format, filename)
    }
    fn filter(&self, channels: &str) {
        self.file.filter(channels)
    }
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self {
        Self {
            filepath: self.filepath.clone(),
            file: self.file.resample(raster, version, time_from_zero),
            channels: Vec::new(),
        }
    }
    // fn select(
    //     &self,
    //     channels: ChannelsType,
    //     record_offset: isize,
    //     raw: bool,
    //     copy_master: bool,
    //     ignore_value2text_conversions: bool,
    //     record_count: isize,
    //     validate: bool,
    // ) -> Vec<Signal> {
    //     self.file.select(
    //         channels,
    //         record_offset,
    //         raw,
    //         copy_master,
    //         ignore_value2text_conversions,
    //         record_count,
    //         validate,
    //     )
    // }
}

pub trait MDFFile {
    fn channels(&self) -> Vec<MdfChannel>;
    fn find_time_channel(
        &self,
        _datagroup: usize,
        channel_grp: usize,
    ) -> Result<usize, &'static str>;

    fn read_channel(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Vec<Record>;

    #[must_use]
    fn new(filepath: &str) -> Self;

    fn read_all(&mut self);

    fn list_data_groups(&mut self);

    fn list_channels(&self);

    #[must_use]
    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> Signal;

    fn cut(&self, start: f64, end: f64, include_ends: bool, time_from_zero: bool);

    fn export(&self, format: &str, filename: &str);
    fn filter(&self, channels: &str);
    #[must_use]
    fn resample(&self, raster: RasterType, version: &str, time_from_zero: bool) -> Self;
    // #[must_use]
    // fn select(
    //     &self,
    //     channels: ChannelsType,
    //     record_offset: isize,
    //     raw: bool,
    //     copy_master: bool,
    //     ignore_value2text_conversions: bool,
    //     record_count: isize,
    //     validate: bool,
    // ) -> Vec<Signal>;
}

pub struct RasterType {}

pub struct ChannelsType {}

pub struct TimeChannel {
    pub time: Vec<f64>,
    pub data: Vec<f64>,
}

impl TimeChannel {
    pub fn new(times: Vec<Record>, datas: Vec<Record>) -> Self {
        let mut t = Vec::with_capacity(times.len());
        let mut d = Vec::with_capacity(datas.len());

        for time in times {
            t.push(time.extract());
        }

        for data in datas {
            d.push(data.extract());
        }

        Self { time: t, data: d }
    }

    pub fn max_time(&self) -> f64 {
        return *self.time.last().expect("Error reading time");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdfChannel {
    pub name: String,
    pub data_group: usize,
    pub channel_group: usize,
    pub channel: usize,
    pub parent_name: Option<String>,
    pub is_nested: bool,
}

impl MdfChannel {
    pub fn full_path(&self) -> String {
        format!(
            "/DG{}/CG{}/{}",
            self.data_group, self.channel_group, self.name
        )
    }

    pub fn full_name(&self) -> String {
        if let Some(parent) = &self.parent_name {
            format!("{}.{}", parent, self.name)
        } else {
            self.name.clone()
        }
    }

    pub fn is_child_of(&self, parent_name: &str) -> bool {
        self.parent_name.as_ref().map_or(false, |p| p == parent_name)
    }
}

#[derive(Debug, Clone)]
pub struct SourceInformation {
    pub source_type: SourceType,
    pub bus_type: BusType,
    pub source_name: String,
    pub source_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const DEMO_FILES: [&str; 4] = [
        "example_files/ASAP2_Demo_V171.mf4",
        "example_files/ASAP2_Demo_V171_deflate.mf4",
        "example_files/ASAP2_Demo_V171_transpose_deflate.mf4",
        "example_files/Discrete_deflate.mf4",
    ];

    const SMALL_DEMO_FILES: [&str; 4] = [
        "example_files/ASAP2_Demo_V171.mf4",
        "example_files/ASAP2_Demo_V171_deflate.mf4",
        "example_files/ASAP2_Demo_V171_transpose_deflate.mf4",
        "example_files/Discrete_deflate.mf4",
    ];

    fn create_test_mdf3_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();

        // Calculate offsets
        let id_block_offset = 0;
        let hd_block_offset = id_block_offset + 64;
        let tx_block_offset = hd_block_offset + 164;
        let dg_block_offset = tx_block_offset + 32;
        let cg_block_offset = dg_block_offset + 28;
        let cn_block_offset = cg_block_offset + 26;

        // ID Block (64 bytes)
        let mut id_block = Vec::new();
        id_block.extend_from_slice(&[b'M', b'D', b'F', b' ', b' ', b' ', b' ', b' ']); // "MDF     "
        id_block.extend_from_slice(&[b'3', b'.', b'3', b'0', b' ', b' ', b' ', b' ']); // "3.30    "
        id_block.extend_from_slice(&[b'r', b's', b'm', b'd', b'f', b' ', b' ', b' ']); // "rsmdf   "
        id_block.extend_from_slice(&[0x00, 0x00]); // default_byte_order (little endian)
        id_block.extend_from_slice(&[0x00, 0x00]); // default_float_format
        id_block.extend_from_slice(&[0x4A, 0x01]); // version_number (330)
        id_block.extend_from_slice(&[0x00, 0x00]); // code_page_number
        id_block.extend_from_slice(&[0x00, 0x00]); // reserved1
        id_block.extend_from_slice(&[0x00; 30]); // reserved2

        // HD Block (164 bytes)
        let mut hd_block = Vec::new();
        hd_block.extend_from_slice(&[b'H', b'D']); // block_type
        hd_block.extend_from_slice(&[0xA4, 0x00, 0x00, 0x00]); // block_size (164)
        hd_block.extend_from_slice(&(dg_block_offset as u32).to_le_bytes().as_slice()); // pointer_dg
        hd_block.extend_from_slice(&(tx_block_offset as u32).to_le_bytes().as_slice()); // pointer_tx
        hd_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // pointer_pr
        hd_block.extend_from_slice(&[0x01, 0x00]); // data_group_number (1)
        hd_block.extend_from_slice(&[b'0', b'1', b'.', b'0', b'1', b'.', b'2', b'4', b' ', b' ']); // date
        hd_block.extend_from_slice(&[b'0', b'0', b':', b'0', b'0', b':', b'0', b'0']); // time
        hd_block.extend_from_slice(&[0x00; 32]); // author
        hd_block.extend_from_slice(&[0x00; 32]); // department
        hd_block.extend_from_slice(&[0x00; 32]); // project
        hd_block.extend_from_slice(&[0x00; 32]); // subject
        hd_block.extend_from_slice(&[0x00; 8]); // timestamp
        hd_block.extend_from_slice(&[0x00, 0x00]); // utc_time_offset
        hd_block.extend_from_slice(&[0x00, 0x00]); // time_quality
        hd_block.extend_from_slice(&[0x00; 32]); // timer_id

        // TX Block (32 bytes)
        let mut tx_block = Vec::new();
        tx_block.extend_from_slice(&[b'T', b'X']); // block_type
        tx_block.extend_from_slice(&[0x20, 0x00, 0x00, 0x00]); // block_size (32)
        tx_block.extend_from_slice(b"Test MDF3 File\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"); // text content

        // DG Block (28 bytes)
        let mut dg_block = Vec::new();
        dg_block.extend_from_slice(&[b'D', b'G']); // block_type
        dg_block.extend_from_slice(&[0x1C, 0x00, 0x00, 0x00]); // block_size (28)
        dg_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // pointer_next_dg
        dg_block.extend_from_slice(&(cg_block_offset as u32).to_le_bytes().as_slice()); // pointer_first_cg
        dg_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // trigger_block
        dg_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // data_block
        dg_block.extend_from_slice(&[0x01, 0x00]); // group_number
        dg_block.extend_from_slice(&[0x01, 0x00]); // id_number
        dg_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // reserved

        // CG Block (26 bytes)
        let mut cg_block = Vec::new();
        cg_block.extend_from_slice(&[b'C', b'G']); // block_type
        cg_block.extend_from_slice(&[0x1A, 0x00, 0x00, 0x00]); // block_size (26)
        cg_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // pointer_next_cg
        cg_block.extend_from_slice(&(cn_block_offset as u32).to_le_bytes().as_slice()); // pointer_first_cn
        cg_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // comment
        cg_block.extend_from_slice(&[0x01, 0x00]); // record_id
        cg_block.extend_from_slice(&[0x01, 0x00]); // number_of_channels
        cg_block.extend_from_slice(&[0x00, 0x00]); // record_size
        cg_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // record_number
        cg_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // first_sample_reduction_block

        // CN Block (228 bytes)
        let mut cn_block = Vec::new();
        cn_block.extend_from_slice(&[b'C', b'N']); // block_type
        cn_block.extend_from_slice(&[0xE4, 0x00, 0x00, 0x00]); // block_size (228)
        cn_block.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // pointer_next_cn
        cn_block.extend_from_slice(&[0x00, 0x00]); // channel_type
        cn_block.extend_from_slice(&[0x00, 0x00]); // short_name_length
        cn_block.extend_from_slice(&[0x00, 0x00]); // description_length
        cn_block.extend_from_slice(&[0x00; 216]); // remaining CN block data

        // Write all blocks to file
        file.write_all(&id_block).unwrap();
        file.write_all(&hd_block).unwrap();
        file.write_all(&tx_block).unwrap();
        file.write_all(&dg_block).unwrap();
        file.write_all(&cg_block).unwrap();
        file.write_all(&cn_block).unwrap();
        file.flush().unwrap();

        file
    }

    #[test]
    fn test_mdf3_version_detection() {
        let test_file = create_test_mdf3_file();
        let version = MDFType::check_version(test_file.path().to_str().unwrap());
        assert_eq!(version, MDFVersion::MDF3);
    }

    #[test]
    fn test_mdf4_version_detection() {
        // Test all demo files
        for file in DEMO_FILES.iter() {
            let version = MDFType::check_version(file);
            assert_eq!(version, MDFVersion::MDF4, "File {} should be MDF4", file);
        }
    }

    #[test]
    fn test_mdf4_file_loading() {
        // Test smaller demo files first
        for file in SMALL_DEMO_FILES.iter() {
            let mut mdf = MDF::new(file);
            assert!(
                !mdf.filepath.is_empty(),
                "File path should not be empty for {}",
                file
            );
            mdf.read_all();
            assert!(
                !mdf.channels.is_empty(),
                "Channels should not be empty for {}",
                file
            );
        }
    }

    #[test]
    fn test_mdf4_channel_listing() {
        // Test smaller demo files first
        for file in SMALL_DEMO_FILES.iter() {
            let mut mdf = MDF::new(file);
            mdf.read_all();
            let channels = mdf.channels();
            assert!(
                !channels.is_empty(),
                "Channels should not be empty for {}",
                file
            );
            println!("Found {} channels in {}", channels.len(), file);
        }
    }

    #[test]
    fn test_mdf4_channel_search() {
        // Test smaller demo files first
        for file in SMALL_DEMO_FILES.iter() {
            let mut mdf = MDF::new(file);
            mdf.read_all();
            let channels = mdf.channels();
            if let Some(first_channel) = channels.first() {
                let channel = mdf.search_channel_first(&first_channel.name);
                assert!(
                    channel.is_ok(),
                    "Should find channel {} in {}",
                    first_channel.name,
                    file
                );
                let found = channel.unwrap();
                assert_eq!(found.name, first_channel.name);
            } else {
                panic!("No channels found in {}", file);
            }
        }
    }

    #[test]
    fn test_mdf4_duplicate_channels() {
        // Test smaller demo files first
        for file in SMALL_DEMO_FILES.iter() {
            let mut mdf = MDF::new(file);
            mdf.read_all();
            let channels = mdf.channels();

            // Create a map to track channel names and their occurrences
            let mut channel_counts = std::collections::HashMap::new();
            for channel in &channels {
                *channel_counts.entry(channel.name.clone()).or_insert(0) += 1;
            }

            // Find channels that appear multiple times
            let duplicate_channels: Vec<_> = channel_counts
                .iter()
                .filter(|(_name, count)| **count > 1)
                .collect();

            if !duplicate_channels.is_empty() {
                for (name, count) in &duplicate_channels {
                    println!("\nFound {} instances of channel '{}'", count, name);

                    // Get all instances of this channel
                    let instances = mdf.search_channels(name);
                    assert_eq!(instances.len(), **count as usize, "Search should find all instances");

                    // Verify each instance has a unique full path
                    let paths: Vec<String> = instances.iter().map(|ch| ch.full_path()).collect();
                    let unique_paths: std::collections::HashSet<_> = paths.iter().collect();
                    assert_eq!(
                        paths.len(),
                        unique_paths.len(),
                        "Each instance should have a unique path"
                    );

                    // Test exact search works for each instance
                    for channel in &instances {
                        let exact_match = mdf.search_channel_exact(
                            &channel.name,
                            channel.data_group,
                            channel.channel_group,
                        );
                        assert!(
                            exact_match.is_some(),
                            "Should find exact match for {}",
                            channel.full_path()
                        );
                        let found = exact_match.unwrap();
                        assert_eq!(found.full_path(), channel.full_path());
                        assert_eq!(found.data_group, channel.data_group);
                        assert_eq!(found.channel_group, channel.channel_group);
                        assert_eq!(found.channel, channel.channel);
                    }
                }
            } else {
                println!("\nNo duplicate channels found in {}", file);
            }
        }
    }

    #[test]
    fn test_mdf4_signal_reading() {
        // Test smaller demo files first
        for file in SMALL_DEMO_FILES.iter() {
            let mut mdf = MDF::new(file);
            mdf.read_all();
            let channels = mdf.channels();

            if let Some(first_channel) = channels.first() {
                let channel = mdf.search_channel_first(&first_channel.name).unwrap();
                let signal = mdf.read_channel(&channel);
                assert!(
                    !signal.samples.is_empty(),
                    "Signal samples should not be empty for {} in {}",
                    first_channel.name,
                    file
                );
                println!(
                    "Successfully read {} samples from channel {} in {}",
                    signal.samples.len(),
                    first_channel.name,
                    file
                );
            } else {
                panic!("No channels found in {}", file);
            }
        }
    }

}
