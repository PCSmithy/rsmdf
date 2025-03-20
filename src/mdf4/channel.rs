use crate::mdf4::{block::Block, md_block::Mdblock};

#[derive(Debug)]
pub struct Channel {
    name: String,
    description: String,
    unit: String,
    data: Vec<f64>,
}

impl Channel {
    pub fn new(name: String, description: String, unit: String) -> Self {
        Self {
            name,
            description,
            unit,
            data: Vec::new(),
        }
    }

    pub fn read_metadata(&mut self, bytes: &[u8], pos: usize) -> Result<usize, String> {
        let (pos, md_block) = Mdblock::read(bytes, pos, true)?;
        let metadata = md_block.data();
        
        // Parse metadata string
        for line in metadata.lines() {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }
            
            match parts[0].trim() {
                "name" => self.name = parts[1].trim().to_string(),
                "description" => self.description = parts[1].trim().to_string(),
                "unit" => self.unit = parts[1].trim().to_string(),
                _ => {}
            }
        }
        
        Ok(pos)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }

    pub fn data(&self) -> &[f64] {
        &self.data
    }

    pub fn add_data(&mut self, value: f64) {
        self.data.push(value);
    }
} 