pub mod loader_tests {
    mod edge_case_tests;
    mod error_tests;
    mod magic_tests;
    mod parse_tests;
    mod version_tests;

    use nvm_core::loader::{NVMLoader, err::LoaderError};

    // Создаёт минимальный валидный .nb-файл с версией 0.1.0.
    pub fn make_nb(data: &[u8]) -> Vec<u8> {
        let mut bytes = b"NVMBC".to_vec();
        bytes.extend_from_slice(&0u16.to_le_bytes()); // major = 0
        bytes.extend_from_slice(&1u16.to_le_bytes()); // minor = 1
        bytes.extend_from_slice(&0u16.to_le_bytes()); // patch = 0
        bytes.extend_from_slice(data);
        bytes
    }

    // Создаёт .nb-файл с произвольной версией.
    pub fn make_nb_with_version(major: u16, minor: u16, patch: u16, data: &[u8]) -> Vec<u8> {
        let mut bytes = b"NVMBC".to_vec();
        bytes.extend_from_slice(&major.to_le_bytes());
        bytes.extend_from_slice(&minor.to_le_bytes());
        bytes.extend_from_slice(&patch.to_le_bytes());
        bytes.extend_from_slice(data);
        bytes
    }

    // Запускает загрузчик и возвращает результат.
    pub fn run_loader(
        data: Vec<u8>,
    ) -> Result<Vec<nvm_core::isa::instruction::Instruction>, LoaderError> {
        NVMLoader::new(data).transpile()
    }

    // Создаёт байты инструкции NOP (опкод 0, 0 операндов).
    pub fn nop_bytes() -> Vec<u8> {
        vec![0x00, 0x00]
    }

    // Создаёт байты инструкции EXIT (опкод 1, 0 операндов).
    pub fn exit_bytes() -> Vec<u8> {
        vec![0x01, 0x00]
    }

    // Создаёт байты инструкции MOVE с двумя регистрами.
    pub fn move_reg_reg(dst: u8, src: u8) -> Vec<u8> {
        vec![0x02, 0x02, 0x00, dst, 0x00, src]
    }

    // Создаёт байты инструкции MOVE с регистром и immediate.
    pub fn move_reg_imm(dst: u8, val: u64) -> Vec<u8> {
        let mut bytes = vec![0x02, 0x02, 0x00, dst, 0x01];
        bytes.extend_from_slice(&val.to_le_bytes());
        bytes
    }
}
