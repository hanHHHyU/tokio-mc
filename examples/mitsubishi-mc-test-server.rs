use std::{
    future,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::net::TcpListener;

use tokio_mc::{
    frame::{ProtocolError, Request, Response},
    server::{
        tcp::{accept_tcp_connection, Server},
        Service,
    },
};

/// 三菱MC协议测试服务器，支持D、X、Y、M、L区域测试
/// 实现了完整的三菱MC协议地址映射和数据格式
/// 每个区域初始化2000个word（4000字节/位）连续内存空间
struct MitsubishiMcTestServer {
    // 使用连续内存存储每个区域的数据
    d_zone: Arc<Mutex<Vec<u8>>>,   // D区域：4000字节，十进制地址
    x_zone: Arc<Mutex<Vec<bool>>>, // X区域：4000个位，十六进制地址
    y_zone: Arc<Mutex<Vec<bool>>>, // Y区域：4000个位，十六进制地址
    m_zone: Arc<Mutex<Vec<bool>>>, // M区域：4000个bool值，十进制地址，M0-M3999
    l_zone: Arc<Mutex<Vec<bool>>>, // L区域：4000个bool值，十进制地址，L0-L3999
}

impl Service for MitsubishiMcTestServer {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ProtocolError;
    type Future = future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let res = match req {
            Request::ReadU8s(ref addr, word_count) => {
                let (zone, start_addr) = parse_address(addr.as_ref());
                log::info!(
                    "Reading {} words ({} bytes) from {} zone, starting at address: {}",
                    word_count,
                    word_count * 2,
                    zone,
                    start_addr
                );

                match zone.as_str() {
                    "D" => {
                        // D区域：从Vec<u8>读取字节数据
                        let zone_data = &self.d_zone;

                        let data = zone_data.lock().unwrap();
                        let bytes_to_read = (word_count as usize) * 2;
                        let byte_offset = start_addr * 2;

                        let mut result = if byte_offset < data.len() {
                            let end_offset = std::cmp::min(byte_offset + bytes_to_read, data.len());
                            data[byte_offset..end_offset].to_vec()
                        } else {
                            log::warn!("Read address {} out of range in {} zone", start_addr, zone);
                            vec![0u8; bytes_to_read]
                        };

                        // 如果读取的字节不足，用0补齐
                        while result.len() < bytes_to_read {
                            result.push(0);
                        }

                        Ok(Response::ReadU8s(result))
                    }
                    "X" | "Y" | "M" | "L" => {
                        // X、Y、M、L区域：从bool数组读取，打包成u16字，返回小端字节序
                        let zone_data = match zone.as_str() {
                            "X" => &self.x_zone,
                            "Y" => &self.y_zone,
                            "M" => &self.m_zone,
                            "L" => &self.l_zone,
                            _ => unreachable!(),
                        };

                        let data = zone_data.lock().unwrap();
                        let mut result = Vec::new();

                        log::info!("Using bool-to-u16 conversion for {} zone", zone);

                        for word_idx in 0..word_count {
                            let bit_start = start_addr + (word_idx as usize) * 16; // 每个字16位

                            // 从bool数组中读取16个位
                            let mut word_value: u16 = 0;
                            for bit_idx in 0..16 {
                                let bit_addr = bit_start + bit_idx;
                                if bit_addr < data.len() && data[bit_addr] {
                                    word_value |= 1 << bit_idx; // 设置对应位
                                }
                            }

                            // 转换为小端字节序
                            let bytes = word_value.to_le_bytes();
                            result.extend_from_slice(&bytes);

                        }

                        log::info!(
                            "Read {} words from {} zone as bytes: {:02X?}",
                            word_count,
                            zone,
                            &result
                        );
                        Ok(Response::ReadU8s(result))
                    }
                    _ => {
                        log::error!("Unknown zone: {}", zone);
                        Ok(Response::ReadU8s(vec![0u8; (word_count as usize) * 2]))
                    }
                }
            }
            Request::WriteU8s(ref addr, ref values) => {
                let (zone, start_addr) = parse_address(addr.as_ref());
                log::info!(
                    "Writing {} bytes to {} zone, starting at address: {} (byte offset: {}): {:?}",
                    values.len(),
                    zone,
                    start_addr,
                    start_addr * 2,
                    values
                );

                // 将字节转换为word值进行显示
                let word_values: Vec<u16> = values
                    .chunks_exact(2)
                    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();
                if !word_values.is_empty() {
                    log::info!("As u16 words: {:?}", word_values);
                }

                match zone.as_str() {
                    "D" => {
                        // D区域：使用标准的字节写入
                        let zone_data = &self.d_zone;

                        let mut data = zone_data.lock().unwrap();
                        let byte_offset = start_addr * 2;

                        if byte_offset < data.len() {
                            let end_offset = std::cmp::min(byte_offset + values.len(), data.len());
                            let bytes_to_write = end_offset - byte_offset;

                            data[byte_offset..end_offset]
                                .copy_from_slice(&values[..bytes_to_write]);
                            log::info!(
                                "Write successful to {} zone starting at address {}",
                                zone,
                                start_addr
                            );

                            if bytes_to_write < values.len() {
                                log::warn!(
                                    "Only wrote {} of {} bytes due to zone boundary",
                                    bytes_to_write,
                                    values.len()
                                );
                            }
                        } else {
                            log::error!(
                                "Write address {} out of range in {} zone",
                                start_addr,
                                zone
                            );
                        }
                    }
                    "X" | "Y" | "M" | "L" => {
                        // X、Y、M、L区域：将u8字节解包成bool数组
                        let zone_data = match zone.as_str() {
                            "X" => &self.x_zone,
                            "Y" => &self.y_zone,
                            "M" => &self.m_zone,
                            "L" => &self.l_zone,
                            _ => unreachable!(),
                        };

                        let mut data = zone_data.lock().unwrap();

                        log::info!("Using u8-to-bool conversion for {} zone", zone);

                        // 将字节转换为u16字，然后解包为bool位
                        for (word_idx, word_bytes) in values.chunks_exact(2).enumerate() {
                            let word_value = u16::from_le_bytes([word_bytes[0], word_bytes[1]]);
                            let bit_start = start_addr + word_idx * 16; // 每个字16位

                            // 将u16字的每一位设置到bool数组中
                            for bit_idx in 0..16 {
                                let bit_addr = bit_start + bit_idx;
                                if bit_addr < data.len() {
                                    let bit_value = (word_value >> bit_idx) & 1 != 0;
                                    data[bit_addr] = bit_value;

                                }
                            }

                            log::info!(
                                "Word {} -> bit_start: {}, u16_value: 0x{:04X}, bytes: [{:02X}, {:02X}]",
                                word_idx, bit_start, word_value, word_bytes[0], word_bytes[1]
                            );
                        }

                        log::info!("Write {} bytes to {} zone as bool bits", values.len(), zone);
                    }
                    _ => {
                        log::error!("Unknown zone: {}", zone);
                        return future::ready(Ok(Response::WriteU8s()));
                    }
                }

                Ok(Response::WriteU8s())
            }
            Request::ReadBits(ref addr, bit_count) => {
                let (zone, start_addr) = parse_address(addr.as_ref());
                log::info!(
                    "Reading {} bits from {} zone, starting at address: {}",
                    bit_count,
                    zone,
                    start_addr
                );

                let mut result_bits = Vec::new();

                match zone.as_str() {
                    "D" => {
                        // D区域：从Vec<u8>读取位数据，使用与字操作相同的地址映射
                        let zone_data = &self.d_zone;

                        let data = zone_data.lock().unwrap();
                        let base_byte_offset = start_addr * 2;
                        log::info!(
                            "Using word-aligned mapping, base byte offset: {}",
                            base_byte_offset
                        );

                        for i in 0..bit_count {
                            // 计算位在字内的偏移 (每个字16位)
                            let bit_in_word = i as usize % 16;
                            // 计算跨越多少个字
                            let word_offset = i as usize / 16;
                            // 最终字节偏移
                            let byte_offset = base_byte_offset + word_offset * 2 + bit_in_word / 8;
                            // 字节内的位偏移
                            let bit_offset = bit_in_word % 8;

                            if byte_offset < data.len() {
                                let byte_value = data[byte_offset];
                                let bit_value = (byte_value >> bit_offset) & 0x01 != 0;
                                result_bits.push(bit_value);
                            } else {
                                result_bits.push(false); // 超出范围返回false
                                log::warn!("Bit {} out of range, byte_offset: {}", i, byte_offset);
                            }
                        }
                    }
                    "X" | "Y" | "M" | "L" => {
                        // X、Y、M、L区域：从Vec<bool>直接读取位数据
                        let zone_data = match zone.as_str() {
                            "X" => &self.x_zone,
                            "Y" => &self.y_zone,
                            "M" => &self.m_zone,
                            "L" => &self.l_zone,
                            _ => unreachable!(),
                        };

                        let data = zone_data.lock().unwrap();
                        log::info!("Using direct bool array access for {} zone", zone);

                        for i in 0..bit_count {
                            let bit_addr = start_addr + i as usize;

                            if bit_addr < data.len() {
                                let bit_value = data[bit_addr];
                                result_bits.push(bit_value);

                            } else {
                                result_bits.push(false); // 超出范围返回false
                                log::warn!("Bit {} out of range, bit_addr: {}", i, bit_addr);
                            }
                        }
                    }
                    _ => {
                        log::error!("Unknown zone: {}", zone);
                        return future::ready(Ok(Response::ReadBits(vec![
                            false;
                            bit_count as usize
                        ])));
                    }
                }

                log::info!(
                    "Read {} bits from {} zone: {:?}",
                    bit_count,
                    zone,
                    &result_bits
                );
                Ok(Response::ReadBits(result_bits))
            }
            Request::WriteBits(ref addr, ref bits) => {
                let (zone, start_addr) = parse_address(addr.as_ref());
                log::info!(
                    "Writing {} bits to {} zone, starting at address: {}: {:?}",
                    bits.len(),
                    zone,
                    start_addr,
                    bits
                );

                match zone.as_str() {
                    "D" => {
                        // D区域：写入Vec<u8>位数据，使用与字操作相同的地址映射
                        let zone_data = &self.d_zone;

                        let mut data = zone_data.lock().unwrap();
                        let base_byte_offset = start_addr * 2;
                        log::info!(
                            "Using word-aligned mapping, base byte offset: {}",
                            base_byte_offset
                        );

                        for (i, &bit_value) in bits.iter().enumerate() {
                            // 计算位在字内的偏移 (每个字16位)
                            let bit_in_word = i % 16;
                            // 计算跨越多少个字
                            let word_offset = i / 16;
                            // 最终字节偏移
                            let byte_offset = base_byte_offset + word_offset * 2 + bit_in_word / 8;
                            // 字节内的位偏移
                            let bit_offset = bit_in_word % 8;

                            if byte_offset < data.len() {
                                let mut byte_value = data[byte_offset];

                                if bit_value {
                                    // 设置位为1
                                    byte_value |= 1 << bit_offset;
                                } else {
                                    // 设置位为0
                                    byte_value &= !(1 << bit_offset);
                                }

                                data[byte_offset] = byte_value;
                            } else {
                                log::warn!("Bit {} out of range, byte_offset: {}", i, byte_offset);
                            }
                        }
                    }
                    "X" | "Y" | "M" | "L" => {
                        // X、Y、M、L区域：直接写入Vec<bool>位数据
                        let zone_data = match zone.as_str() {
                            "X" => &self.x_zone,
                            "Y" => &self.y_zone,
                            "M" => &self.m_zone,
                            "L" => &self.l_zone,
                            _ => unreachable!(),
                        };

                        let mut data = zone_data.lock().unwrap();
                        log::info!("Using direct bool array access for {} zone", zone);

                        for (i, &bit_value) in bits.iter().enumerate() {
                            let bit_addr = start_addr + i;

                            if bit_addr < data.len() {
                                let old_value = data[bit_addr];
                                data[bit_addr] = bit_value;
                            } else {
                                log::warn!("Bit {} out of range, bit_addr: {}", i, bit_addr);
                            }
                        }
                    }
                    _ => {
                        log::error!("Unknown zone: {}", zone);
                        return future::ready(Ok(Response::WriteBits()));
                    }
                }

                log::info!(
                    "Write {} bits successful to {} zone starting at address {}",
                    bits.len(),
                    zone,
                    start_addr
                );
                Ok(Response::WriteBits())
            }
        };
        future::ready(res)
    }
}

impl MitsubishiMcTestServer {
    fn new() -> Self {
        log::info!("正在初始化三菱MC协议测试服务器...");

        // D区域和M区域：每个区域初始化2000个word（4000字节）
        let zone_size = 2000 * 2; // 2000 words × 2 bytes per word = 4000 bytes

        log::info!("Initializing D zone with {} bytes...", zone_size);
        let d_zone = Arc::new(Mutex::new(vec![0u8; zone_size]));

        // X区域和Y区域：每个区域初始化4000个位
        let bit_zone_size = 4000; // 4000 bits

        log::info!("Initializing X zone with {} bits...", bit_zone_size);
        let x_zone = Arc::new(Mutex::new(vec![false; bit_zone_size]));

        log::info!("Initializing Y zone with {} bits...", bit_zone_size);
        let y_zone = Arc::new(Mutex::new(vec![false; bit_zone_size]));

        log::info!("Initializing M zone with {} bits...", bit_zone_size);
        let m_zone = Arc::new(Mutex::new(vec![false; bit_zone_size]));

        log::info!("Initializing L zone with {} bits...", bit_zone_size);
        let l_zone = Arc::new(Mutex::new(vec![false; bit_zone_size]));

        log::info!("三菱MC协议测试服务器初始化成功！");
        log::info!("支持区域总数: 5 (D, X, Y, M, L)");
        log::info!("D zone: 0-1999 words (4000 bytes)");
        log::info!("X zone: X0-X3999 bits (4000 bits)");
        log::info!("Y zone: Y0-Y3999 bits (4000 bits)");
        log::info!("M zone: M0-M3999 bits (4000 bits)");
        log::info!("L zone: L0-L3999 bits (4000 bits)");

        Self {
            d_zone,
            x_zone,
            y_zone,
            m_zone,
            l_zone,
        }
    }

    /// 打印指定区域的状态统计（处理不同类型的区域）
    fn print_zone_status_u8(&self, zone_name: &str, zone_data: &Arc<Mutex<Vec<u8>>>) {
        let data = zone_data.lock().unwrap();
        let non_zero_count = data.iter().filter(|&&b| b != 0).count();
        let total_bytes = data.len();

        log::info!(
            "{} zone status: {}/{} bytes have non-zero data",
            zone_name,
            non_zero_count,
            total_bytes
        );

        // 显示前几个非零位置的示例
        let mut non_zero_positions = Vec::new();
        for (i, &byte) in data.iter().enumerate().take(20) {
            if byte != 0 {
                non_zero_positions.push((i, byte));
            }
        }

        if !non_zero_positions.is_empty() {
            log::info!("  First few non-zero bytes: {:?}", non_zero_positions);
        }
    }

    /// 打印bool区域的状态统计
    fn print_zone_status_bool(&self, zone_name: &str, zone_data: &Arc<Mutex<Vec<bool>>>) {
        let data = zone_data.lock().unwrap();
        let true_count = data.iter().filter(|&&b| b).count();
        let total_bits = data.len();

        log::info!(
            "{} zone status: {}/{} bits are true",
            zone_name,
            true_count,
            total_bits
        );

        // 显示前几个true位置的示例
        let mut true_positions = Vec::new();
        for (i, &bit) in data.iter().enumerate().take(20) {
            if bit {
                true_positions.push(i);
            }
        }

        if !true_positions.is_empty() {
            log::info!("  First few true bit positions: {:?}", true_positions);
        }
    }

    /// 打印所有区域状态
    fn print_all_status(&self) {
        log::info!("=== 三菱MC协议测试服务器状态报告 ===");
        self.print_zone_status_u8("D", &self.d_zone);
        self.print_zone_status_bool("X", &self.x_zone);
        self.print_zone_status_bool("Y", &self.y_zone);
        self.print_zone_status_bool("M", &self.m_zone);
        self.print_zone_status_bool("L", &self.l_zone);
    }
}

/// 解析地址字符串，返回(zone, address_number)
/// 例如: "D5" -> ("D", 5), "X100" -> ("X", 256), "X10" -> ("X", 16), "L20" -> ("L", 20)
fn parse_address(addr: &str) -> (String, usize) {
    if addr.is_empty() {
        return ("Unknown".to_string(), 0);
    }

    let zone = addr.chars().next().unwrap().to_string().to_uppercase();
    let addr_num_str = &addr[1..];

    // 根据区域类型使用不同的进制解析
    let addr_num = match zone.as_str() {
        "X" | "Y" => {
            // X和Y区域使用16进制
            u32::from_str_radix(addr_num_str, 16).unwrap_or(0) as usize
        }
        _ => {
            // D、M、L区域使用10进制
            addr_num_str.parse::<usize>().unwrap_or(0)
        }
    };

    (zone, addr_num)
}

/// 测试L区域的功能
async fn test_l_zone() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("=== Testing L zone functionality ===");

    let service = MitsubishiMcTestServer::new();

    // 测试写入 L100 = -1 (0xFFFF)
    let write_addr = "L100";
    let i16_value = -1i16;
    let bytes = i16_value.to_le_bytes().to_vec(); // [0xFF, 0xFF]
    log::info!(
        "Writing i16 value {} to L100 as bytes: {:02X?}",
        i16_value,
        bytes
    );

    let write_result = service
        .call(Request::WriteU8s(write_addr.into(), bytes.into()))
        .await?;
    log::info!("Write result: {:?}", write_result);

    // 打印L区域状态
    service.print_zone_status_bool("L", &service.l_zone);

    // 测试读取L100的位数据
    let read_addr = "L100";
    log::info!("Reading 16 bits from L100");
    let read_result = service
        .call(Request::ReadBits(read_addr.into(), 16))
        .await?;

    if let Response::ReadBits(bits) = read_result {
        log::info!("L100 16-bit pattern: {:?}", bits);
        let all_true = bits.iter().all(|&b| b);
        log::info!("All bits are true: {} (expected: true)", all_true);

        if !all_true {
            log::error!("ERROR: All L100 bits should be true but some are false!");
        }
    }

    // 测试读取L100的字数据
    let read_u8_result = service.call(Request::ReadU8s(read_addr.into(), 1)).await?;
    if let Response::ReadU8s(bytes) = read_u8_result {
        log::info!("L100 as bytes: {:02X?} (expected: [FF, FF])", bytes);
        if bytes != vec![0xFF, 0xFF] {
            log::error!(
                "ERROR: L100 bytes should be [FF, FF] but got {:02X?}!",
                bytes
            );
        }
    }

    log::info!("L zone test completed successfully!");
    Ok(())
}

/// 测试X1写入和XF/XB读取的问题
async fn test_x1_write_xf_read() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("=== Testing X1 write and XF/XB read issue ===");

    let service = MitsubishiMcTestServer::new();

    // 测试写入 X1 = -1 (0xFFFF)
    let write_addr = "X1";
    let i16_value = -1i16;
    let bytes = i16_value.to_le_bytes().to_vec(); // [0xFF, 0xFF]
    log::info!(
        "Writing i16 value {} to X1 as bytes: {:02X?}",
        i16_value,
        bytes
    );

    let write_result = service
        .call(Request::WriteU8s(write_addr.into(), bytes.into()))
        .await?;
    log::info!("Write result: {:?}", write_result);

    // 打印X区域状态
    service.print_zone_status_bool("X", &service.x_zone);

    // 测试读取XF位 (十六进制F = 15)
    let read_addr_f = "XF";
    log::info!("Reading bit at address XF (hex F = decimal 15)");
    let read_result_f = service
        .call(Request::ReadBits(read_addr_f.into(), 1))
        .await?;

    if let Response::ReadBits(bits) = read_result_f {
        log::info!("XF bit value: {} (expected: true)", bits[0]);
        if !bits[0] {
            log::error!("ERROR: XF should be true but got false!");
        }
    }

    // 测试读取XB位 (十六进制B = 11)
    let read_addr_b = "XB";
    log::info!("Reading bit at address XB (hex B = decimal 11)");
    let read_result_b = service
        .call(Request::ReadBits(read_addr_b.into(), 1))
        .await?;

    if let Response::ReadBits(bits) = read_result_b {
        log::info!("XB bit value: {} (expected: true)", bits[0]);
        if !bits[0] {
            log::error!("ERROR: XB should be true but got false!");
        }
    }

    // 详细分析：显示X1地址对应的16个位的状态
    log::info!("=== Detailed Analysis ===");
    log::info!(
        "Address X1 maps to decimal address: {}",
        parse_address("X1").1
    );
    log::info!(
        "Address XF maps to decimal address: {}",
        parse_address("XF").1
    );
    log::info!(
        "Address XB maps to decimal address: {}",
        parse_address("XB").1
    );

    // 读取X1的16个位
    let read_x1_bits = service.call(Request::ReadBits("X1".into(), 16)).await?;
    if let Response::ReadBits(bits) = read_x1_bits {
        log::info!("X1 16-bit pattern: {:?}", bits);
        for (i, &bit) in bits.iter().enumerate() {
            log::info!("  Bit {} (addr X{:X}): {}", i, 1 + i, bit);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    // 首先运行测试用例
    if let Err(e) = test_x1_write_xf_read().await {
        log::error!("X1/XF test failed: {}", e);
        return Err(e);
    }

    // 测试L区域功能
    if let Err(e) = test_l_zone().await {
        log::error!("L zone test failed: {}", e);
        return Err(e);
    }

    let socket_addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();

    tokio::select! {
        result = server_context(socket_addr) => {
            if let Err(e) = result {
                log::error!("Server error: {}", e);
            }
        },
        _ = client_info() => println!("Client info completed"),
    }

    Ok(())
}

async fn server_context(socket_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("=== 启动三菱MC协议TCP测试服务器 ===");
    log::info!("Server listening on: {}", socket_addr);
    log::info!("Supported zones: D, X, Y, M, L (each with 2000 addresses, continuous memory)");
    log::info!("You can test this server with:");
    log::info!("  cargo run --example multi-zone-client-test");

    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);

    let service = Arc::new(MitsubishiMcTestServer::new());

    // 每10秒打印一次服务器状态
    let status_service = Arc::clone(&service);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            status_service.print_all_status();
        }
    });

    let on_connected = {
        let service = Arc::clone(&service);
        move |stream, socket_addr| {
            let service = Arc::clone(&service);
            async move {
                log::info!("New connection established from: {}", socket_addr);
                accept_tcp_connection(stream, socket_addr, move |_| Ok(Some(Arc::clone(&service))))
            }
        }
    };

    let on_process_error = |err| {
        log::error!("Connection process error: {}", err);
    };

    log::info!("Server ready and waiting for connections...");
    server.serve(&on_connected, on_process_error).await?;
    Ok(())
}

async fn client_info() {
    // 给服务器一些启动时间
    tokio::time::sleep(Duration::from_secs(3)).await;

    log::info!("=== 三菱MC协议测试服务器信息 ===");
    log::info!("此服务器支持5个不同区域的连续内存测试:");
    log::info!("  D Zone: D0 - D1999 (Data registers, 4000 bytes continuous)");
    log::info!("  X Zone: X0 - X1999 (Input registers, 4000 bytes continuous)");
    log::info!("  Y Zone: Y0 - Y1999 (Output registers, 4000 bytes continuous)");
    log::info!("  M Zone: M0 - M1999 (Memory registers, 4000 bytes continuous)");
    log::info!("  L Zone: L0 - L1999 (Link registers, 4000 bytes continuous)");
    log::info!("");
    log::info!("Memory model: Continuous byte array per zone");
    log::info!("Address mapping: Address N maps to byte offset N*2 in zone");
    log::info!("Example: D5 maps to bytes 10-11 in D zone, D4-D5 read gives bytes 8-11");
    log::info!("");
    log::info!("Example operations:");
    log::info!("  - Write [0x12, 0x00] to D5: Writes to bytes 10-11 in D zone");
    log::info!("  - Read 2 words from D4: Reads bytes 8-11 from D zone (should be [0x00, 0x00, 0x12, 0x00])");

    // 保持程序运行
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        log::info!("三菱MC协议测试服务器运行中... 按Ctrl+C停止");
    }
}
