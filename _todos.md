# Matrix-iMessage Bridge 开发任务列表

## Phase 1: 基础项目结构 ✅
- [x] 创建项目目录结构
- [x] 创建 Cargo.toml
- [x] 创建 README.md
- [x] 创建 main.rs 入口文件
- [x] 创建 utils 模块
- [x] 创建 config 模块骨架

## Phase 2: 配置系统 ✅
- [x] 实现配置解析器 (config/parser.rs)
- [x] 实现配置验证器 (config/validator.rs)
- [x] 创建配置示例文件 (config/config.sample.yaml)
- [x] 支持 iMessage 平台配置（mac, mac-nosip, bluebubbles）

## Phase 3: 数据库层 ✅
- [x] 设计数据库 schema (db/schema.rs)
- [x] 实现数据库管理器 (db/manager.rs)
- [x] 实现消息映射模型 (db/models.rs)
- [x] 实现房间映射模型
- [x] 实现用户映射模型
- [x] 支持 SQLite
- [x] 支持 PostgreSQL (基础结构)
- [x] 支持 MySQL (基础结构)
- [ ] 实现数据库迁移 (待完善)

## Phase 4: iMessage API 客户端 ✅
- [x] 定义 iMessage API 接口 (imessage/interface.rs)
- [x] 定义数据结构 (imessage/structs.rs)
  - [x] Message 消息结构
  - [x] Contact 联系人结构
  - [x] ChatInfo 聊天信息结构
  - [x] Attachment 附件结构
  - [x] ReadReceipt 读回执
  - [x] TypingNotification 输入通知
  - [x] Tapback 反应类型
- [x] 实现 BlueBubbles 连接器 (imessage/bluebubbles/)
- [x] 实现 mac-nosip 连接器 (imessage/mac_nosip/)
- [ ] 实现消息发送功能 (待完善)
- [ ] 实现消息接收功能 (待完善)
- [ ] 实现文件发送/接收 (待完善)
- [ ] 实现读回执 (待完善)
- [ ] 实现输入状态通知 (待完善)

## Phase 5: Matrix 集成 ✅
- [x] 实现 Matrix AppService 客户端 (matrix/appservice.rs)
- [x] 实现事件处理器 (matrix/event_handler.rs)
- [x] 实现命令处理器 (matrix/command_handler.rs)
- [x] 实现用户管理
- [x] 实现房间管理
- [x] 实现消息发送
- [x] 实现消息接收和处理

## Phase 6: Bridge 核心逻辑 ✅
- [x] 实现 BridgeCore 核心结构 (bridge/core.rs)
- [x] 实现消息流转逻辑 (bridge/message_flow.rs)
- [x] 实现用户同步 (bridge/user_sync.rs)
- [x] 实现消息桥接逻辑 (bridge/logic.rs)
- [x] 实现消息队列 (bridge/queue.rs)
- [x] 实现状态管理 (bridge/presence_handler.rs)
- [x] 实现配置供应 (bridge/provisioning.rs)

## Phase 7: 消息类型支持
### Matrix → iMessage
- [ ] 纯文本消息
- [ ] 媒体/文件消息
- [ ] 回复消息
- [ ] 反应 (Tapbacks)
- [ ] 消息编辑
- [ ] 消息撤回
- [ ] 读回执
- [ ] 输入通知

### iMessage → Matrix
- [ ] 纯文本消息
- [ ] 媒体/文件消息
- [ ] 回复消息
- [ ] Tapbacks 反应
- [ ] 消息编辑
- [ ] 消息撤回
- [ ] 读回执
- [ ] 输入通知
- [ ] 用户元数据
- [ ] 群组元数据
- [ ] 群成员变更事件

## Phase 8: 媒体处理
- [ ] 实现媒体处理器 (media.rs)
- [ ] 支持 HEIF 图片格式转换
- [ ] 支持文件上传/下载
- [ ] 支持 MIME 类型检测
- [ ] 实现媒体缓存

## Phase 9: Web 管理界面
- [ ] 实现 Web 服务器 (web/server.rs)
- [ ] 实现健康检查接口
- [ ] 实现状态查询接口
- [ ] 实现管理接口

## Phase 10: 其他功能
- [ ] 实现命令行接口 (cli.rs)
- [ ] 实现管理命令 (admin.rs)
- [ ] 实现缓存系统 (cache.rs)
- [ ] 实现富链接预览
- [ ] 实现消息回填功能
- [ ] 实现自动创建门户房间
- [ ] 实现双聊合并功能

## Phase 11: 部署和文档
- [ ] 创建 Dockerfile
- [ ] 创建 docker-compose.yml
- [ ] 编写部署文档
- [ ] 编写配置说明
- [ ] 编写开发指南
- [ ] 添加单元测试
- [ ] 添加集成测试

## Phase 12: 优化和完善
- [ ] 性能优化
- [ ] 错误处理完善
- [ ] 日志完善
- [ ] 监控指标
- [ ] 安全加固
