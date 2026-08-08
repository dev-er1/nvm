pub mod loader_tests {
    mod edge_case_tests;
    mod error_tests;
    mod magic_tests;
    mod parse_tests;
    mod version_tests;

    use nvm_core::loader::{NVMLoader, err::LoaderError};

    // Creates a minimal valid .nb-file with version 0.1.0.
    pub fn make_nb(data: &[u8]) -> Vec<u8> {
        let mut bytes = b"NVMBC".to_vec();
        bytes.extend_from_slice(&0u16.to_le_bytes()); // major = 0
        bytes.extend_from_slice(&1u16.to_le_bytes()); // minor = 1
        bytes.extend_from_slice(&0u16.to_le_bytes()); // patch = 0
        bytes.extend_from_slice(data);
        bytes
    }

    // Creates a .nb-file with an arbitrary version.
    pub fn make_nb_with_version(major: u16, minor: u16, patch: u16, data: &[u8]) -> Vec<u8> {
        let mut bytes = b"NVMBC".to_vec();
        bytes.extend_from_slice(&major.to_le_bytes());
        bytes.extend_from_slice(&minor.to_le_bytes());
        bytes.extend_from_slice(&patch.to_le_bytes());
        bytes.extend_from_slice(data);
        bytes
    }

    // Runs the loader and returns the result.
    pub fn run_loader(
        data: Vec<u8>,
    ) -> Result<Vec<nvm_core::isa::instruction::Instruction>, LoaderError> {
        NVMLoader::new(data).transpile()
    }

    // Creates bytes of the NOP instruction (opcode 0, 0 operands).
    pub fn nop_bytes() -> Vec<u8> {
        vec![0x00, 0x00]
    }

    // Creates bytes of the EXIT instruction (opcode 1, 0 operands).
    pub fn exit_bytes() -> Vec<u8> {
        vec![0x01, 0x00]
    }

    // Creates bytes of the MOVE instruction with two registers.
    pub fn move_reg_reg(dst: u8, src: u8) -> Vec<u8> {
        vec![0x02, 0x02, 0x00, dst, 0x00, src]
    }

    // Creates bytes of the MOVE instruction with a register and an immediate.
    pub fn move_reg_imm(dst: u8, val: u64) -> Vec<u8> {
        let mut bytes = vec![0x02, 0x02, 0x00, dst, 0x01];
        bytes.extend_from_slice(&val.to_le_bytes());
        bytes
    }
}
