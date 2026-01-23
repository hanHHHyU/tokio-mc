# tokio-mc 测试驱动开发 (TDD) 文档

> 本项目采用 TDD 开发模式，本文档记录测试规范和实践指南。

---

## 目录

1. [概述](#1-概述)
2. [测试现状](#2-测试现状)
3. [测试架构](#3-测试架构)
4. [测试规范](#4-测试规范)
5. [Mock 与 Fake 策略](#5-mock-与-fake-策略)
6. [命令速查](#6-命令速查)

---

## 1. 概述

### 1.1 TDD 原则

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│    ┌─────────┐     ┌─────────┐     ┌─────────────┐     │
│    │  🔴 红  │ ──▶ │  🟢 绿  │ ──▶ │  🔄 重构   │     │
│    │ 写失败  │     │ 写代码  │     │  改进代码  │     │
│    │  测试   │     │ 使通过  │     │  保持通过  │     │
│    └─────────┘     └─────────┘     └─────────────┘     │
│         ▲                                   │          │
│         └───────────────────────────────────┘          │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 1.2 开发流程

**新功能开发**:
1. 先编写失败的测试
2. 运行测试，确认失败（红色）
3. 编写最少代码使测试通过
4. 运行测试，确认通过（绿色）
5. 重构代码，保持测试通过
6. 提交代码

**Bug 修复**:
1. 编写失败测试（重现 bug）
2. 修复代码
3. 确认测试通过
4. 提交代码

---

## 2. 测试现状

### 2.1 测试统计

```
测试总数: 52 ✅ 全部通过
├── 客户端模块 (client)     : 24 tests ✅
├── 编解码模块 (codec)      : 15 tests ✅
├── 帧协议模块 (frame)      : 11 tests ✅
└── 服务端模块 (server)     :  2 tests ✅
```

### 2.2 覆盖率

```
模块                     覆盖率
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
client/mod.rs            ██████████████████░░ 70% ✅
client/tcp.rs            ████████████████░░░░ 80% ✅
codec/mod.rs             ████████████████████ 80% ✅
codec/tcp.rs             █████████████████░░░ 85% ✅
frame/mod.rs             ██████████░░░░░░░░░░ 50% ✅
frame/map.rs             ████████████░░░░░░░░ 60% ✅
frame/kv/*               ███████████░░░░░░░░░ 55% ✅
server/service.rs        ████████░░░░░░░░░░░░ 40% ✅
server/tcp.rs            ███████████████░░░░░ 75% ✅
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
整体估计                 ██████████████░░░░░░ ~65%
```

---

## 3. 测试架构

### 3.1 测试金字塔

```
                    ▲
                   /│\
                  / │ \        端到端测试 (E2E) ~5%
                 /  │  \       - 完整通信测试
                /───┼───\
               /    │    \     集成测试 ~20%
              /     │     \    - 模块间交互
             /──────┼──────\
            /       │       \  单元测试 ~75%
           /        │        \ - 编码/解码/地址转换
          /─────────┼─────────\
```

### 3.2 目录结构

```
tokio-mc/
├── src/
│   ├── client/
│   │   ├── mod.rs         # 内联测试 (mod context_tests)
│   │   └── tcp.rs         # 内联测试 (mod tcp_tests)
│   ├── codec/
│   │   ├── mod.rs         # 内联测试 (mod tests)
│   │   └── tcp.rs         # 内联测试 (mod tests)
│   ├── frame/
│   │   └── *.rs           # 各模块内联测试
│   └── server/
│       ├── service.rs     # 内联测试
│       └── tcp.rs         # 内联测试
│
├── tests/                  # 集成测试
│   ├── common/
│   │   ├── mod.rs         # 测试工具函数
│   │   ├── mock_client.rs # MockClient 实现
│   │   └── fake_plc.rs    # FakePlc 模拟器
│   ├── client_tests.rs
│   ├── protocol_tests.rs
│   └── server_tests.rs
│
└── examples/              # 示例程序
```

---

## 4. 测试规范

### 4.1 命名约定

```rust
// 格式: test_<被测函数>_<场景>_<预期结果>
#[test]
fn test_encode_request_with_valid_address_returns_bytes() { }

#[test]
fn test_decode_response_with_invalid_header_returns_error() { }
```

### 4.2 AAA 模式

```rust
#[test]
fn test_read_u16s_converts_bytes_correctly() {
    // Arrange (准备)
    let bytes = vec![0x34, 0x12, 0x78, 0x56];

    // Act (执行)
    let result = bytes_to_u16s(&bytes);

    // Assert (断言)
    assert_eq!(result, vec![0x1234, 0x5678]);
}
```

### 4.3 异步测试模板

```rust
#[tokio::test]
async fn test_async_operation() {
    // Arrange
    let mock = MockClient::new()
        .expect_response(Response::ReadU8s(vec![0x12, 0x34]));
    let mut ctx = Context::new(mock);

    // Act
    let result = ctx.read_u8s("D100", 2).await;

    // Assert
    assert_eq!(result.unwrap(), vec![0x12, 0x34]);
    ctx.client.verify();
}
```

### 4.4 边界条件测试

```rust
#[cfg(test)]
mod boundary_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let result = process_data(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_max_capacity() {
        let data = vec![0u8; LIMIT as usize];
        let result = process_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_exceeds_limit() {
        let data = vec![0u8; (LIMIT + 1) as usize];
        let result = process_data(&data);
        assert!(result.is_err());
    }
}
```

---

## 5. Mock 与 Fake 策略

### 5.1 MockClient

用于单元测试，模拟客户端响应：

```rust
use crate::test_common::MockClient;

#[tokio::test]
async fn test_with_mock_client() {
    // 创建 mock，设置期望响应
    let mock = MockClient::new()
        .expect_response(Response::ReadU8s(vec![0x12, 0x34]));

    let mut ctx = Context::new(mock);

    // 执行测试
    let result = ctx.read_u8s("D100", 2).await.unwrap();

    // 验证
    assert_eq!(result, vec![0x12, 0x34]);
    ctx.client.verify();  // 验证所有期望都被满足
}
```

**MockClient API**:
- `MockClient::new()` - 创建新实例
- `.expect_response(response)` - 添加预期响应
- `.requests()` - 获取收到的请求列表
- `.verify()` - 验证所有期望都已满足

### 5.2 FakePlc

用于集成测试，模拟 PLC 设备：

```rust
use crate::test_common::FakePlc;

#[tokio::test]
async fn test_with_fake_plc() {
    let plc = FakePlc::new();

    // 预设寄存器值
    plc.preset_d_register(100, 0x1234);
    plc.preset_m_bit(0, true);

    // 用于服务端测试...
}
```

**FakePlc API**:
- `FakePlc::new()` - 创建新实例
- `.preset_d_register(addr, value)` - 预设 D 寄存器
- `.preset_m_bit(addr, value)` - 预设 M 位
- `.preset_x_input(addr, value)` - 预设 X 输入
- `.preset_y_output(addr, value)` - 预设 Y 输出

### 5.3 测试辅助函数

```rust
use crate::test_common::create_test_listener;

#[tokio::test]
async fn test_tcp_connection() {
    // 创建测试用 TCP 监听器（随机端口）
    let (listener, addr) = create_test_listener().await;

    // 使用 addr 进行连接测试...
}
```

---

## 6. 命令速查

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test client::
cargo test codec::
cargo test frame::

# 显示详细输出
cargo test -- --nocapture

# 运行单个测试
cargo test test_context_read_u8s

# 运行特定功能测试
cargo test --features "3e-async"
cargo test --features "3e-sync"
cargo test --features "server"

# 代码质量检查
cargo clippy --all-features
cargo fmt --check

# 快速编译检查
cargo check
```

---

## 附录: 测试清单

### 已完成测试 ✅

| 模块 | 测试 | 状态 |
|------|------|------|
| client/mod.rs | test_context_read_u8s | ✅ |
| client/mod.rs | test_context_read_u16s | ✅ |
| client/mod.rs | test_context_read_i16s | ✅ |
| client/mod.rs | test_context_read_u32s | ✅ |
| client/mod.rs | test_context_read_i32s | ✅ |
| client/mod.rs | test_context_read_f32s | ✅ |
| client/mod.rs | test_context_read_u64s | ✅ |
| client/mod.rs | test_context_read_i64s | ✅ |
| client/mod.rs | test_context_read_f64s | ✅ |
| client/mod.rs | test_context_read_bools | ✅ |
| client/mod.rs | test_context_write_* (10个) | ✅ |
| client/mod.rs | test_context_process_address_* | ✅ |
| client/tcp.rs | test_tcp_connect_* | ✅ |
| client/tcp.rs | test_tcp_client_* | ✅ |
| codec/mod.rs | test_encode/decode (13个) | ✅ |
| codec/tcp.rs | test_* (2个) | ✅ |
| frame/* | test_* (11个) | ✅ |
| server/* | test_* (2个) | ✅ |

---

*文档版本: 1.0.0 | 更新日期: 2025-12-30*
