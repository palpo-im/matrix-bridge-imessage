# Matrix-iMessage Bridge 开发任务列表

## ✅ 已完成的基础阶段

### Phase 1-6: 核心架构 (100%)
- [x] 基础项目结构
- [x] 配置系统（YAML 解析、验证）
- [x] 数据库层（SQLite/PostgreSQL/MySQL 支持）
- [x] iMessage API 客户端接口定义
- [x] Matrix 集成（AppService、事件处理）
- [x] Bridge 核心逻辑（消息流转、用户同步）

### Phase 7-11: 基础功能 (80%)
- [x] 基本消息桥接
- [x] 平台连接器框架
- [x] 部署支持（Docker、文档）

---

## 🔴 Phase 13: 高优先级 - 核心消息处理（2-3周）

### 13.1 消息处理主循环
- [ ] 实现 iMessageHandler 主循环 (参考: references/imessage/imessage.go:41-90)
  - [ ] 消息接收 handler
  - [ ] 消息处理队列
  - [ ] 错误处理和重试机制
  - [ ] 优雅关闭

### 13.2 Portal 管理
- [ ] 实现 Portal 创建和管理 (参考: references/imessage/portal.go:52-228)
  - [ ] Portal 数据结构完善
  - [ ] Portal 创建逻辑
  - [ ] Portal 同步逻辑
  - [ ] Portal 删除逻辑
  - [ ] Portal 缓存机制

### 13.3 Tapback/Reaction 系统
- [ ] 实现 Tapback 双向桥接 (参考: references/imessage/portal.go:1512-1600)
  - [ ] Matrix → iMessage 反应
  - [ ] iMessage → Matrix Tapback
  - [ ] Tapback 移除处理
  - [ ] 数据库模型 (tapback 表)

### 13.4 Backfill 历史同步系统
- [ ] 实现消息回填功能 (参考: references/imessage/historysync.go)
  - [ ] Backfill 队列管理
  - [ ] 分块回填逻辑
  - [ ] 回填状态追踪
  - [ ] 数据库模型 (backfill 表)
  - [ ] backfillqueue.go 实现

### 13.5 命令系统完善
- [ ] 实现完整命令系统 (参考: references/imessage/commands.go)
  - [ ] `pm <contact>` - 发起私信
  - [ ] `search-contacts <query>` - 搜索联系人
  - [ ] `refresh-contacts` - 刷新联系人列表
  - [ ] `merge <chat-id>` - 合并聊天
  - [ ] `unmerge <chat-id>` - 拆分聊天

### 13.6 Web 管理界面完善
- [ ] 实现 Web API (参考: matrix-bridge-discord/web/*.rs)
  - [ ] `/health` - 健康检查
  - [ ] `/status` - 状态查询
  - [ ] `/metrics` - Prometheus 指标
  - [ ] `/api/v1/provisioning` - 供应接口

### 13.7 数据库模型完善
- [ ] 添加缺失的数据库表 (参考: references/imessage/database/*.go)
  - [ ] `tapback` 表 - 反应记录
  - [ ] `backfill` 表 - 回填状态
  - [ ] `merged_chat` 表 - 合并聊天
  - [ ] `kv_store` 表 - 键值存储
  - [ ] 数据库迁移脚本

---

## 🟡 Phase 14: 中优先级 - 双向同步（2周）

### 14.1 Puppet 系统
- [ ] 实现用户傀儡系统 (参考: references/imessage/puppet.go)
  - [ ] Puppet 创建和管理
  - [ ] Puppet 资料同步
  - [ ] Custom Puppet (自定义傀儡)
  - [ ] Puppet 缓存

### 14.2 聊天合并功能
- [ ] 实现聊天合并/拆分 (参考: references/imessage/chatmerging.go)
  - [ ] 基于联系人的聊天合并
  - [ ] 合并状态管理
  - [ ] 拆分逻辑
  - [ ] 合并历史同步

### 14.3 IPC 系统
- [ ] 实现进程间通信 (参考: references/imessage/ipc/ipc.go)
  - [ ] IPC 消息定义
  - [ ] IPC 客户端
  - [ ] IPC 服务器
  - [ ] 消息路由

### 14.4 Websocket 命令
- [ ] 实现 Websocket 管理 (参考: references/imessage/matrix.go:38-455)
  - [ ] `start_dm` - 开始私信
  - [ ] `create_group` - 创建群组
  - [ ] `edit_ghost` - 编辑傀儡
  - [ ] 其他管理命令

### 14.5 Bridge Status
- [ ] 实现状态上报 (参考: references/imessage/main.go:345-423)
  - [ ] Bridge 状态追踪
  - [ ] Push Key 机制
  - [ ] 状态上报到 Matrix
  - [ ] 健康检查

### 14.6 消息状态事件
- [ ] 实现消息状态通知 (参考: references/imessage/portal.go:1010-1156)
  - [ ] 送达确认 (DELIVERED)
  - [ ] 失败通知 (FAILED)
  - [ ] 发送中 (SENDING)
  - [ ] 状态事件发送到 Matrix

### 14.7 Read Receipt 完善
- [ ] 完善已读回执 (参考: references/imessage/portal.go:449-502, 1457-1475)
  - [ ] Matrix → iMessage 已读回执
  - [ ] iMessage → Matrix 已读回执
  - [ ] 批量已读回执处理
  - [ ] 已读回执缓存

### 14.8 Typing Notification 完善
- [ ] 完善输入状态 (参考: references/imessage/portal.go:1477-1510)
  - [ ] Matrix → iMessage 输入状态
  - [ ] iMessage → Matrix 输入状态
  - [ ] 输入状态超时处理
  - [ ] 输入状态节流

### 14.9 消息缓存
- [ ] 实现消息缓存 (参考: matrix-bridge-discord/bridge.rs:171-191)
  - [ ] Room 缓存 (TTL 15分钟)
  - [ ] Message 缓存
  - [ ] 缓存失效策略
  - [ ] 缓存命中率优化

### 14.10 Presence Handler
- [ ] 实现在线状态处理 (参考: matrix-bridge-discord/bridge/presence_handler.rs)
  - [ ] 用户在线状态同步
  - [ ] 状态变更事件
  - [ ] 批量状态更新
  - [ ] 状态缓存

---

## 🟢 Phase 15: 低优先级 - 增强功能（2-3周）

### 15.1 媒体转换
- [ ] 实现媒体格式转换 (参考: references/imessage/heif.go, tiff.go)
  - [ ] HEIF → JPEG/PNG 转换
  - [ ] TIFF → JPEG/PNG 转换
  - [ ] 图片质量配置
  - [ ] 转换缓存

### 15.2 视频转码
- [ ] 实现视频转码 (参考: references/imessage/config/bridge.go:77-82)
  - [ ] FFMPEG 集成
  - [ ] 视频格式转换
  - [ ] 视频压缩
  - [ ] 缩略图生成

### 15.3 URL Preview 完善
- [ ] 完善富链接预览 (参考: references/imessage/urlpreview.go)
  - [ ] Open Graph 解析
  - [ ] Twitter Card 解析
  - [ ] 预览缓存
  - [ ] 预览大小限制

### 15.4 Media Viewer
- [ ] 实现媒体查看器 (参考: references/imessage/mediaviewer.go)
  - [ ] 大文件媒体查看
  - [ ] 临时 URL 生成
  - [ ] 访问控制
  - [ ] 过期机制

### 15.5 Segment 分析
- [ ] 实现使用分析 (参考: references/imessage/segment.go)
  - [ ] Segment SDK 集成
  - [ ] 事件追踪
  - [ ] 用户行为分析
  - [ ] 隐私保护

### 15.6 权限检查
- [ ] 实现 Mac 权限检查 (参考: references/imessage/mac-permissions.go)
  - [ ] 完全磁盘访问权限检查
  - [ ] 联系人权限检查
  - [ ] 权限请求提示
  - [ ] 权限状态报告

### 15.7 Sleep Detection
- [ ] 实现睡眠检测 (参考: references/imessage/mac/sleepdetect.go)
  - [ ] Mac 睡眠事件监听
  - [ ] 唤醒后重连逻辑
  - [ ] 状态保存和恢复
  - [ ] 通知用户

### 15.8 Hacky Startup Test
- [ ] 实现启动连接测试 (参考: references/imessage/connecttest.go)
  - [ ] 启动时连接测试
  - [ ] 失败重试机制
  - [ ] 错误报告
  - [ ] 降级模式

---

## 📋 Phase 16: 测试和文档（持续）

### 16.1 测试
- [ ] 单元测试
  - [ ] 配置解析测试
  - [ ] 数据库操作测试
  - [ ] 消息转换测试
  - [ ] 工具函数测试
- [ ] 集成测试
  - [ ] 消息桥接测试
  - [ ] 数据库迁移测试
  - [ ] API 接口测试
- [ ] 端到端测试
  - [ ] 完整消息流程测试
  - [ ] 多平台测试

### 16.2 文档
- [ ] API 文档
  - [ ] OpenAPI 规范
  - [ ] API 使用示例
  - [ ] 错误码说明
- [ ] 用户文档
  - [ ] 安装指南
  - [ ] 配置说明
  - [ ] 故障排查
- [ ] 开发文档
  - [ ] 架构说明
  - [ ] 贡献指南
  - [ ] 发布流程

---

## 🎯 实施计划

### 第 1-2 周：Phase 13.1-13.3
- 消息处理主循环
- Portal 管理
- Tapback 系统

### 第 3 周：Phase 13.4-13.5
- Backfill 系统
- 命令系统

### 第 4 周：Phase 13.6-13.7
- Web 管理界面
- 数据库完善

### 第 5-6 周：Phase 14.1-14.5
- Puppet 系统
- 聊天合并
- IPC 系统
- Websocket 命令
- Bridge Status

### 第 7-8 周：Phase 14.6-14.10
- 消息状态
- Read Receipt
- Typing Notification
- 消息缓存
- Presence Handler

### 第 9-11 周：Phase 15
- 媒体转换
- 视频转码
- URL Preview
- Media Viewer
- 其他增强功能

### 持续：Phase 16
- 测试
- 文档
- 优化

---

## 📊 进度追踪

- **Phase 1-6**: ✅ 100% (已完成)
- **Phase 7-11**: ✅ 80% (基础功能已完成)
- **Phase 13**: ⏳ 0% (待开始)
- **Phase 14**: ⏳ 0% (待开始)
- **Phase 15**: ⏳ 0% (待开始)
- **Phase 16**: ⏳ 0% (待开始)

**总体进度**: 约 40% 完成
