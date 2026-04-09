# mac_traffic_monitor

一个面向 macOS 菜单栏的轻量网络流量监控工具。

它会在菜单栏中实时显示：
- 上传 / 下载速率
- CPU 占用
- 内存占用

点击菜单栏项后，可以查看并复制更多详细信息：
- 本机 IPv4 / IPv6
- 外网 IPv4 / IPv6
- 子网掩码
- 默认网关
- 累计流量
- 自程序启动以来流量
- 程序运行时长

## 功能特性

- 原生 macOS 菜单栏状态视图
- 实时网络速率采样与平滑显示
- 本机网络信息展示
- 外网 IP 自动获取
- 菜单项点击复制
- 开机自启动开关
- 自定义应用图标

## 技术栈

- Rust
- Cocoa / Objective-C runtime
- sysinfo
- tao

## 运行环境

- macOS
- Rust stable

## 本地运行

```bash
cargo run
```

## 构建

```bash
cargo build --release
```

## 打包 .app

项目内置了 macOS `.app` 打包脚本：

```bash
./scripts/package_app.sh
```

执行后会生成：

- `dist/mac_traffic_monitor.app`
- `dist/mac_traffic_monitor-macos-app.zip`

其中 zip 文件适合直接上传到 GitHub Releases。

## 安装与使用

### 从 GitHub Releases 下载

1. 打开仓库的 **Releases** 页面
2. 下载：`mac_traffic_monitor-macos-app.zip`
3. 解压得到：`mac_traffic_monitor.app`
4. 将 `.app` 拖到 `Applications` 文件夹，或直接双击运行

### 首次打开提示已损坏 / 无法验证开发者

因为当前版本可能还未进行 Apple 签名与 notarization，macOS 首次打开时可能拦截。

可以尝试：

- 在 Finder 中右键应用 → **打开**
- 或前往：**系统设置 → 隐私与安全性**，允许打开该应用

## 发布到 GitHub

建议在 GitHub Release 中上传这个文件：

- `dist/mac_traffic_monitor-macos-app.zip`

推荐发布流程：

1. 更新版本号
2. 运行打包脚本
3. 创建 GitHub Release（例如 `v0.1.0`）
4. 上传 zip 文件
5. 填写更新说明

## 项目结构

```text
src/
├── main.rs                # 启动入口与事件编排
└── app/
    ├── autostart.rs       # 开机自启动
    ├── constants.rs       # 常量
    ├── format.rs          # 文本格式化
    ├── icon.rs            # 应用图标加载
    ├── monitor.rs         # 监控采样与速率计算
    ├── network.rs         # 本机/外网网络信息查询
    ├── types.rs           # 核心数据结构
    └── ui/
        ├── actions.rs     # 菜单动作与复制逻辑
        ├── menu.rs        # 菜单构建
        ├── mod.rs
        └── status_item.rs # 菜单栏状态视图
```

## 测试

```bash
cargo test
```

## 说明

当前版本主要面向 macOS 使用，依赖系统命令与原生 Cocoa 接口，因此暂不支持跨平台。
