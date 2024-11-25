use std::{collections::HashMap, sync::OnceLock};

use super::types::DataOProcess;

// 创建一个基恩士的对于三菱的指令的映射表
type KVMap = HashMap<&'static str, (&'static str, DataOProcess)>;

static KV_MAP: OnceLock<KVMap> = OnceLock::new();

fn kv_map_data() -> Vec<(&'static str, &'static str, DataOProcess)> {
    vec![
        ("R", "X", DataOProcess::Hex),
        ("MR", "M", DataOProcess::Decimal),
        ("LR", "L", DataOProcess::Decimal),
        ("DM", "D", DataOProcess::None),
        ("FM", "R", DataOProcess::None),
        ("B", "B", DataOProcess::None),
        ("ZF", "ZR", DataOProcess::DecimalToHex),
        // XYM标记
        ("M", "M", DataOProcess::None),
        ("D", "D", DataOProcess::None),
        ("F", "R", DataOProcess::None),
        ("L", "L", DataOProcess::None),
        // 特殊
        ("X", "X", DataOProcess::XYToHex),
        ("Y", "Y", DataOProcess::XYToHex),
    ]
}

pub fn find(prefix: &str) -> Option<(&'static str, DataOProcess)> {
    KV_MAP
        .get_or_init(|| {
            let mut map = HashMap::new();
            for (key, value, process) in kv_map_data() {
                map.insert(key, (value, process));
            }
            map
        })
        .get(prefix)
        .map(|&(value, process)| (value, process))
}