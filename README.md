# tokio-mc

[![Crates.io](https://img.shields.io/crates/v/tokio-mc.svg)](https://crates.io/crates/tokio-mc)
[![Docs.rs](https://docs.rs/tokio-mc/badge.svg)](https://docs.rs/tokio-mc)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**tokio-mc** is a pure Rust library for Mitsubishi Communication (MC) protocol, built on top of [tokio](https://tokio.rs/).  
**tokio-mc** 是一个基于 [tokio](https://tokio.rs/) 的纯 Rust 三菱通信协议库。

---

## Features 功能

- **Async** communication with Mitsubishi PLCs.  
  与三菱 PLC 进行异步通信。
- Full support for MC protocol commands.  
  完整支持 MC 协议命令。
- Easy integration with `tokio` ecosystem.  
  简单集成到 `tokio` 生态系统。

---

## Installation 安装

Add the following to your `Cargo.toml`:  
将以下内容添加到您的 `Cargo.toml` 文件中：

```toml
[dependencies]
tokio-mc = "0.1.0"
