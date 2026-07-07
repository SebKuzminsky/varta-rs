#[derive(Debug, Clone)]
pub struct VartaEasyblade {
    pub node_id: u8,
    pub serial_number: u16,
    // pub identity: Option<LssIdentity>,
    // pub device_name: Option<String>,
    pub software_version: Option<String>,
    pub hardware_version: Option<String>,
    pub last_seen: std::time::Instant,
    // pub nmt_state: Option<NmtState>,
}
