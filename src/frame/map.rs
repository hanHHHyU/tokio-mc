use std::{collections::HashMap, sync::OnceLock};

use super::{NumberBase, PlcInstruction};

type PlcOperationCodeMap = HashMap<&'static str, PlcInstruction>;

static PLC_OPERATION_CODE_MAP: OnceLock<PlcOperationCodeMap> = OnceLock::new();

fn get_plc_operation_code_map() -> &'static PlcOperationCodeMap {
    PLC_OPERATION_CODE_MAP.get_or_init(|| {
        let data = [
            ("X", 0x9c, NumberBase::Hexadecimal),
            ("Y", 0x9d, NumberBase::Hexadecimal),
            ("F", 0x93, NumberBase::Decimal),
            ("M", 0x90, NumberBase::Decimal),
            ("L", 0x92, NumberBase::Decimal),
            ("D", 0xa8, NumberBase::Decimal),
            ("R", 0xaf, NumberBase::Decimal),
            ("B", 0xA0, NumberBase::Hexadecimal),
            ("SM", 0x91,NumberBase::Decimal), // 特殊继电器
            ("SD", 0xA9,NumberBase::Decimal), // 特殊存储器
            ("ZR", 0xB0,NumberBase::Hexadecimal), // 文件寄存器
            ("W", 0xB4,NumberBase::Hexadecimal), // 链路寄存器
            ("TN", 0xC2,NumberBase::Decimal), // 定时器当前值
            ("TS", 0xC1,NumberBase::Decimal), // 定时器接点
            ("CN", 0xC5, NumberBase::Decimal), // 计数器当前值
            ("CS", 0xC4, NumberBase::Decimal), // 计数器接点
                                                                 // Add other entries...
        ];

        let mut map = PlcOperationCodeMap::new();
        for &(prefix, code, number_base) in &data {
            map.insert(
                prefix,
                PlcInstruction {
                    code,
                    number_base,
                },
            );
        }
        map
    })
}

pub fn find_instruction_code(prefix: &str) -> Option<(u8, NumberBase)> {
    get_plc_operation_code_map().get(prefix).map(|instruction| {
        (
            instruction.code,
            instruction.number_base,
        )
    })
}


pub fn convert_to_base(s: &str, number_base: NumberBase)->Result<u32, std::num::ParseIntError> {
    match number_base {
        NumberBase::Decimal => s.parse::<u32>(),
        NumberBase::Hexadecimal => u32::from_str_radix(s, 16),
    }
}