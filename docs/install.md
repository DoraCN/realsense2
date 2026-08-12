# Realsense Rust 开发环境安装文档

## 1. 环境概览

- 操作系统: Ubuntu 20/22/24 LTS (x86_64 Linux)
- 工具链: Rust 1.97.1 (stable)
- 相机 SDK: librealsense2
- Rust 封装: `realsense-rust`

> 注意: 本机当前 **未安装** librealsense2，且 **未连接** Realsense 相机（`lsusb` 中仅有 Orbbec Gemini 335 等设备）。
>
> **官方不支持在虚拟机中运行**：USB 3.0 透传层会破坏相机通信。如必须在 VM 中测试，官方推荐 VMware Workstation Player（而非 VirtualBox）。请务必使用官方支持列表中的内核版本。

## 2. 安装方式选择

| 方式 | 场景 | 是否需要 patch 内核 |
|---|---|---|
| A. apt 官方仓库（推荐，简单） | 快速搭建开发环境 | 不需要（deb 自动带补丁驱动） |
| B. 从源码编译（官方主推） | 自定义 cmake 选项 / 无预编译包 | 需要（必须 patch uvcvideo 内核模块） |

官方仓库已迁移至: <https://github.com/realsenseai/librealsense>

---

## 方式 A: apt 官方仓库安装（推荐）

```bash
# 1. 注册公钥
sudo mkdir -p /etc/apt/keyrings
curl -sSf https://librealsense.intel.com/Debian/librealsense.pgp | \
  sudo tee /etc/apt/keyrings/librealsense.pgp > /dev/null

# 2. 添加软件源（根据系统版本选择）
echo "deb [signed-by=/etc/apt/keyrings/librealsense.pgp] https://librealsense.intel.com/Debian/apt-repo noble main" | \
  sudo tee /etc/apt/sources.list.d/librealsense.list

# 3. 安装（dkms 包含补丁后的内核模块，utils 含 rs-enumerate-devices 等工具）
sudo apt-get update
sudo apt-get install -y librealsense2-dkms librealsense2-utils librealsense2-dev

# 4. 验证
realsense-viewer
```

> 若同机既有 deb 安装又手动源码安装，会出现 `Multiple realsense udev-rules were found!` 报错，需移除其一。

---

## 方式 B: 从源码编译（官方主推，需 patch 内核）

### 2.1 前置依赖

```bash
sudo apt-get update && sudo apt-get upgrade

# 构建核心依赖
sudo apt-get install libssl-dev libusb-1.0-0-dev libudev-dev pkg-config libgtk-3-dev

# 构建工具链
sudo apt-get install git wget cmake build-essential

# OpenGL 后端（构建带 GUI 的 examples 才需要；headless 部署可跳过）
sudo apt-get install libglfw3-dev libgl1-mesa-dev libglu1-mesa-dev
```

> 说明:
> - `libudev-dev` 可选但推荐——装了之后 SDK 用事件驱动方式检测 USB 设备，否则退化为轮询。
> - cmake 某些选项（如 CUDA）需要 3.8+，apt 的版本可能不满足。
> - `librealsense2e` 核心库与多数工具支持无 GUI 环境。

### 2.2 获取源码 + udev 权限

```bash
git clone https://github.com/realsenseai/librealsense.git
cd librealsense

# 设置 udev 规则（先拔掉相机）
./scripts/setup_udev_rules.sh
# 卸载：./scripts/setup_udev_rules.sh --uninstall
```

### 2.3 patch 并编译内核模块（关键步骤）

深度相机在 Linux 上必须使用打过补丁的 `uvcvideo` 内核模块：

```bash
# Ubuntu 20/22/24 使用 LTS HWE 内核 (5.15/5.19/6.5/6.8/6.11/6.14)
./scripts/patch-realsense-ubuntu-lts-hwe.sh
# Ubuntu 20 且内核 < 5.13 用：
# ./scripts/patch-realsense-ubuntu-lts.sh

# 验证补丁模块已加载
sudo dmesg | tail -n 50    # 应看到注册了新的 uvcvideo 驱动
```

> 若 `uvcvideo` 补丁失败，脚本会恢复原始模块。检查 `uname -r` 内核版本是否在官方支持列表内。
>
> 有的 OEM/厂商会锁定内核禁止修改，需要进 BIOS 解锁。

### 2.4 cmake 编译并安装 SDK

```bash
mkdir build && cd build

# 基础构建（Release 优化）
cmake ../ -DCMAKE_BUILD_TYPE=Release

# 构建官方 demos/examples（可选）
cmake ../ -DCMAKE_BUILD_TYPE=Release -DBUILD_EXAMPLES=true
# 无 OpenGL/X11 环境只构建文本示例：
cmake ../ -DBUILD_EXAMPLES=true -DBUILD_GRAPHICAL_EXAMPLES=false

# 编译并安装
sudo make uninstall && make clean && make && sudo make install
# 多核并行: make -j$(($(nproc)-1)) install

# 验证
realsense-viewer   # 安装到 /usr/local/bin
```

> 安装位置: 共享库在 `/usr/local/lib`，头文件在 `/usr/local/include`，二进制工具在 `/usr/local/bin`。

### 2.5 常见编译选项

| 选项 | 说明 |
|---|---|
| `-DBUILD_WITH_DDS=OFF` | fastrtps/fastcdr 依赖构建失败时关闭 DDS |
| `-DCHECK_FOR_UPDATES=OFF` | 构建失败报 curl 依赖问题时关闭自动更新检查 |
| `-DBUILD_GRAPHICAL_EXAMPLES=false` | headless 环境跳过 OpenGL 示例 |

---

## 3. NVIDIA Jetson（Tegra 内核）专用说明

> 判断：`uname -r` 输出含 `tegra`（如 `5.15.148-tegra`，对应 JetPack 5.x / L4T 35.x）即为 Jetson 平台。

**注意：桌面脚本 `patch-realsense-ubuntu-lts-hwe.sh` 在 Jetson 上不可用**——它用 `apt-get install linux-headers-$(uname -r)`，而 Tegra 内核头文件不随 apt 分发，会报
`E: Unable to locate package linux-headers-5.15.148-tegra`。

Jetson 上三种选择：

### 3.1 RSUSB 后端（推荐，免内核 patch）

用户态 UVC/HID 协议栈，不需要改内核，最适合 Jetson。代价：不支持多相机、部分性能/功能受限（如帧元数据）。

```bash
cd librealsense
./scripts/setup_udev_rules.sh
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release \
  -DFORCE_RSUSB_BACKEND=true \
  -DBUILD_WITH_CUDA=true
make -j$(($(nproc)-1)) && sudo make install
```

> 若不想装 CUDA dev-kit，去掉 `-DBUILD_WITH_CUDA=true`。也可参考 `scripts/libuvc_installation.sh`。

### 3.2 L4T 专用内核 patch（原生 V4L2 后端）

> 一句话总结：**在 Jetson 上想用原生后端，必须跑 `patch-realsense-ubuntu-L4T.sh`，而不是桌面的 `patch-realsense-ubuntu-lts-hwe.sh`。** 后者在 Jetson 上必然报 `linux-headers-*.tegra` 找不到，因为 Tegra 内核头文件不随 apt 分发。

#### 为什么不能用桌面脚本？

| | 桌面 x86 (`lts-hwe`) | Jetson (`L4T`) |
|---|---|---|
| 获取内核源码方式 | `apt-get install linux-headers-$(uname -r)` | 脚本自动从 L4T 拉取 kernel source tree |
| 结果 | 直接用 Ubuntu 提供的头文件编译模块 | 先下载 Tegra 内核源码 → 打补丁 → 重编译模块 |
| 在 Jetson 上 | ❌ 报 `Unable to locate package linux-headers-5.15.148-tegra` | ✅ 正常工作 |

#### 前置条件

```bash
# 1. 确认板型和 JetPack 版本（官方验证过的组合）
#    Jetson AGX Orin + JetPack 6.0/6.1/6.2/7.0
#    Jetson AGX Xavier + JetPack 5.0.2
uname -r                                    # 应输出 5.x-tegra 或 6.x-tegra

# 2. 确认磁盘空闲空间（patch 过程需 ~2.5GB）
df -h

# 3. 将 Jetson 切到最大功耗模式（Max power mode，右上角设置）
# 4. 拔掉所有 USB/UVC 相机
```

#### 执行 L4T patch 脚本

```bash
cd librealsense
./scripts/patch-realsense-ubuntu-L4T.sh
```

脚本大约运行 **30 分钟**（取决于网络），依次完成：
1. 拉取与当前内核匹配的 Tegra kernel source tree
2. 应用 Librealsense 专属内核补丁
3. 编译修改后的内核模块
4. 尝试将新模块插入当前内核（失败会自动恢复原始模块）

#### 验证补丁是否生效

```bash
sudo dmesg | tail -n 50
# 应看到注册了新的 uvcvideo 驱动（uvcvideo: Found UVC 1.50 device ...）
```

> 若 `uvcvideo: module verification failed` 是内核 4.4-30+ 的标准告警，**不影响功能**。
> 若脚本失败且 `uname -r` 不在上表组合中，先核对板型/JetPack 版本再重跑。

#### 编译 SDK（显式指定原生后端）

```bash
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_EXAMPLES=true \
  -DFORCE_RSUSB_BACKEND=false \
  -DBUILD_WITH_CUDA=true
make -j$(($(nproc)-1)) && sudo make install && sudo ldconfig
```

#### RSUSB 后端 vs 原生后端的取舍

| | RSUSB 后端（3.1） | 原生 V4L2 后端（3.2） |
|---|---|---|
| 内核 patch | 不需要 | 必须（L4T 脚本） |
| 安装耗时 | ~10 分钟 | ~40 分钟（含 patch） |
| 多相机 | ❌ 不支持 | ✅ 支持 |
| 帧元数据/完整功能 | ⚠️ 部分受限 | ✅ 完整 |
| 官方定位 | 原型验证/新环境 | 生产环境 |

> 官方建议：**生产环境用原生后端**。先用 3.1 RSUSB 跑通链路，需要多相机/完整功能时再升级到 3.2。

### 3.3 直接装 Debian 包（JetPack ≥ 5.0.2）

```bash
sudo apt-get install librealsense2-utils librealsense2-dev
realsense-viewer
```

## 4. 安装 Rust 工具链（如缺失）

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version && cargo --version
```

## 5. 验证 SDK 链路

安装完成后按顺序验证：

```bash
# 1. 注册共享库（源码方式装到 /usr/local，make install 不会自动执行）
sudo ldconfig
ldconfig -p | grep realsense
# 应输出: librealsense2.so.2.58 (libc6,x86-64) => /usr/local/lib/...

# 2. 验证 pkg-config（注意包名是 realsense2，不是 librealsense2）
pkg-config --modversion realsense2     # 输出: 2.58.3
pkg-config --cflags --libs realsense2  # 输出编译链接参数
```

> 若 `pkg-config` 找不到，需：
> ```bash
> export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig
> ```

```bash
# 3. 确认工具已安装（在 /usr/local/bin）
which rs-enumerate-devices realsense-viewer

# 4. 枚举设备（相机已插入时应列出序列号/固件/分辨率；未插则提示无设备）
rs-enumerate-devices
```

## 6. 常见问题

| 问题 | 原因 / 解决 |
|---|---|
| `No devices found` | 相机未插入、USB3 口供电不足、udev 规则未生效、未 patch 内核模块（Jetson 需 RSUSB 后端或 L4T 补丁） |
| 运行时 `error while loading shared libraries: librealsense2.so.2.58` | `make install` 后未执行 `sudo ldconfig` |
| `pkg-config: no package found` | 包名是 `realsense2`；或源码安装需 `export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig` |
| `Multiple realsense udev-rules were found!` | deb 与源码安装并存，移除其一 |
| `uvcvideo: module verification failed` | 内核 4.4-30+ 的标准告警，**不影响功能** |
| 内核模块未加载 / 编译失败 | 内核版本与脚本不匹配，`uname -r` 核对后重来 |
| `gcc: internal compiler error` | 内存/swap 不足，建议 ≥2GB |
| 编译找不到 `librealsense2` | SDK 未安装或 `PKG_CONFIG_PATH` 未指向安装路径 |
| 相机被其他驱动抢占 | 多相机共存时在 Config 中指定设备序列号 |

## 7. 参考链接

- 官方源码安装指南: <https://github.com/realsenseai/librealsense/blob/master/doc/installation.md>
- 官方 apt 分发指南: <https://github.com/realsenseai/librealsense/blob/master/doc/distribution_linux.md>
- 官方 Jetson 安装指南: <https://github.com/realsenseai/librealsense/blob/master/doc/installation_jetson.md>
- realsense-rust crate: <https://crates.io/crates/realsense-rust>
- 构建配置选项: <https://github.com/realsenseai/librealsense/wiki/Build-Configuration>
