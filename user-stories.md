# Claude Watch 测试用户故事

## 项目概述

**项目**: claude-watch - Claude Code 监控系统  
**目标**: 为 Claude Code 项目制定全面的测试用户故事  
**范围**: 覆盖所有核心功能和边界条件  

## Epic 1: 核心活动检测功能

### Story US-001: 标准执行条格式检测
**As a** Claude Code 监控系统  
**I want** 准确识别 Claude Code 的标准执行条格式  
**So that** 能够正确判断 Claude Code 是否正在工作  

**Acceptance Criteria** (EARS format):
- **WHEN** 系统接收到包含 `*(状态)… (时间 · tokens · esc to interrupt)` 格式的文本 **THEN** 系统应该返回 `true` 表示活动状态
- **IF** 文本包含 `* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)` **THEN** 应该正确识别为活动状态
- **FOR** 所有标准状态关键词 (`Cogitating`, `Thinking`, `Herding`, `Meandering`, `Reticulating`) **VERIFY** 系统能够正确识别

**Technical Notes**:
- 需要测试正则表达式 `r"\*[^)]*\([^)]*\d+s[^)]*tokens[^)]*esc to interrupt\)"` 的准确性
- 测试数据应包含真实 Claude Code 输出样本
- 注意区分 `Done` 状态（不应被认为是活动状态）

**Story Points**: 5  
**Priority**: High  

### Story US-002: 工具调用状态检测
**As a** 监控系统  
**I want** 识别 Claude Code 的工具调用状态  
**So that** 避免在工具执行过程中误判为卡住  

**Acceptance Criteria**:
- **WHEN** 文本包含 `Tool use:`, `Calling tool:`, `Function call:` **THEN** 系统应该识别为活动状态
- **IF** 文本包含文件操作关键词 (`Reading file`, `Writing file`, `Creating file`, `Editing file`) **THEN** 应该识别为活动状态
- **FOR** 工具调用的各种变体和格式 **VERIFY** 系统具有足够的识别能力

**Technical Notes**:
- 工具调用检测主要检查最后 15 行文本
- 需要测试中英文混合的工具调用描述
- 考虑工具调用的不同阶段（开始、执行、结束）

**Story Points**: 3  
**Priority**: High  

### Story US-003: 进度指示器检测
**As a** 活动检测系统  
**I want** 识别各种进度指示器  
**So that** 能够准确判断 Claude Code 的处理进度  

**Acceptance Criteria**:
- **WHEN** 文本包含字符进度指示器 (`▪▪▪`, `◦◦◦`, `>>>`, `***`) **THEN** 系统应该识别为活动状态
- **IF** 文本包含省略号结尾 (`...`) **THEN** 应该识别为未完成状态
- **FOR** 不同类型的进度指示器 **VERIFY** 系统的识别准确率

**Technical Notes**:
- 进度指示器检测用于补充执行条格式检测
- 需要测试各种特殊字符和符号组合
- 注意避免误判正常的文本内容

**Story Points**: 3  
**Priority**: Medium  

## Epic 2: 时间检测和状态管理

### Story US-004: 时间提取功能
**As a** 监控系统  
**I want** 从复杂文本中准确提取执行时间  
**So that** 能够追踪 Claude Code 的工作进度  

**Acceptance Criteria**:
- **WHEN** 文本包含 `(数字s)` 格式 **THEN** 系统应该能够提取出数字部分
- **IF** 文本为 `* Herding… (343s · ↑ 14.2k tokens · esc to interrupt)` **THEN** 应该提取出 `343`
- **FOR** 各种时间格式和上下文 **VERIFY** 提取函数的鲁棒性

**Technical Notes**:
- 使用正则表达式 `r"\((\d+)s\)"` 进行时间提取
- 需要测试边界情况（无效格式、缺失括号等）
- 考虑性能优化（避免重复编译正则表达式）

**Story Points**: 3  
**Priority**: High  

### Story US-005: 时间递增检测
**As a** 状态管理系统  
**I want** 检测时间是否在递增  
**So that** 判断 Claude Code 是否真正在工作  

**Acceptance Criteria**:
- **WHEN** 同一窗格的时间值从 343s 增加到 344s **THEN** 系统应该判断为时间递增
- **IF** 时间值保持不变或减少 **THEN** 应该判断为时间未递增
- **FOR** 多个独立窗格 **VERIFY** 系统能够独立追踪每个窗格的时间状态

**Technical Notes**:
- 使用全局静态变量 `TIME_TRACKER` 存储时间状态
- 需要处理线程安全问题（static mut）
- 测试多窗格并发访问场景

**Story Points**: 5  
**Priority**: High  

### Story US-006: 实质性进展检测
**As a** 智能监控系统  
**I want** 区分真正的活动恢复和虚假的时间计数器变化  
**So that** 避免误判和频繁的 LLM 调用  

**Acceptance Criteria**:
- **WHEN** 检测到新的工具调用或思考状态 **THEN** 系统应该判断为有实质性进展
- **IF** 只有时间计数器变化但没有其他内容 **THEN** 应该判断为无实质性进展
- **FOR** 各种进展类型（思考、工具、编译、错误）**VERIFY** 判断的准确性

**Technical Notes**:
- 需要检查最近 5 行文本的实质性变化
- 区分 `* 104s`（纯时间计数器）和 `* Herding… (104s · tokens)`（有实质性内容）
- 考虑长文本输出的性能影响

**Story Points**: 5  
**Priority**: High  

## Epic 3: 智能判断和 LLM 集成

### Story US-007: 跳过 LLM 调用判断
**As a** 智能监控系统  
**I want** 智能判断是否应该跳过 LLM 调用  
**So that** 避免不必要的 LLM 调用和误判  

**Acceptance Criteria**:
- **WHEN** 检测到明确的中断状态 (`Interrupted by user`) **THEN** 系统不应该跳过 LLM 调用
- **IF** 检测到标准执行条格式且时间在递增 **THEN** 应该跳过 LLM 调用
- **FOR** 各种中间状态（思考、编译、工具调用）**VERIFY** 跳过判断的准确性

**Technical Notes**:
- 这是防止误判的关键函数
- 需要测试启发式判断逻辑的准确性
- 考虑命令提示符状态的特殊处理

**Story Points**: 8  
**Priority**: High  

### Story US-008: LLM 状态判断
**As a** 状态判断系统  
**I want** 使用 LLM 准确判断 Claude Code 的最终状态  
**So that** 在画面长时间无变化时做出正确决策  

**Acceptance Criteria**:
- **WHEN** 调用 LLM 服务 **THEN** 系统应该得到准确的 DONE/STUCK 判断
- **IF** LLM 服务不可用 **THEN** 应该回退到启发式检查
- **FOR** 不同的 LLM 后端 (Ollama, OpenAI, OpenRouter) **VERIFY** 集成的正确性

**Technical Notes**:
- 需要解决 Tokio Runtime 冲突问题
- 测试异步调用的正确性
- 考虑网络超时和错误处理

**Story Points**: 8  
**Priority**: High  

### Story US-009: 智能激活功能
**As a** 恢复系统  
**I want** 使用 LLM 生成智能激活消息  
**So that** 能够自然地激活卡住的 Claude Code  

**Acceptance Criteria**:
- **WHEN** Claude Code 被判断为卡住 **THEN** 系统应该生成友好的激活消息
- **IF** LLM 激活失败 **THEN** 应该回退到传统的 Retry 命令
- **FOR** 不同的卡住场景 **VERIFY** 激活消息的有效性

**Technical Notes**:
- 激活消息需要自然、友好，避免负面词汇
- 测试消息长度控制（10-20个字）
- 考虑激活效果的验证机制

**Story Points**: 5  
**Priority**: Medium  

## Epic 4: 配置和参数管理

### Story US-010: 配置文件管理
**As a** 配置管理系统  
**I want** 灵活地加载和管理配置  
**So that** 用户可以自定义监控参数  

**Acceptance Criteria**:
- **WHEN** 系统启动时 **THEN** 应该能够加载配置文件或使用默认配置
- **IF** 配置文件不存在或格式错误 **THEN** 应该使用默认配置并给出提示
- **FOR** 不同的配置参数（LLM 后端、监控参数、tmux 窗格）**VERIFY** 配置的正确性

**Technical Notes**:
- 支持 YAML 格式的配置文件
- 测试配置验证和默认值处理
- 考虑环境变量和命令行参数的优先级

**Story Points**: 5  
**Priority**: Medium  

### Story US-011: 命令行参数解析
**As a** 用户接口  
**I want** 通过命令行参数配置系统  
**So that** 用户可以快速启动和定制监控  

**Acceptance Criteria**:
- **WHEN** 用户提供命令行参数 **THEN** 系统应该正确解析并应用这些参数
- **IF** 参数缺失或格式错误 **THEN** 应该使用默认值并给出错误提示
- **FOR** 所有必需的参数类型 **VERIFY** 解析的正确性

**Technical Notes**:
- 使用 clap 库进行参数解析
- 测试参数类型验证和转换
- 考虑帮助信息生成

**Story Points**: 3  
**Priority**: Medium  

## Epic 5: Tmux 交互和通信

### Story US-012: Tmux 内容捕获
**As a** 数据采集系统  
**I want** 准确捕获 tmux 窗格的内容  
**So that** 能够分析 Claude Code 的状态  

**Acceptance Criteria**:
- **WHEN** 调用捕获函数 **THEN** 系统应该能够获取指定窗格的完整内容
- **IF** tmux 窗格不存在或不可访问 **THEN** 应该给出明确的错误信息
- **FOR** 不同类型的窗格标识符 **VERIFY** 捕获功能的正确性

**Technical Notes**:
- 使用 `tmux capture-pane -p -t {pane}` 命令
- 测试不同窗格标识符格式（%0, session:window.pane）
- 考虑大文本内容的性能影响

**Story Points**: 3  
**Priority**: Medium  

### Story US-013: Tmux 命令发送
**As a** 控制系统  
**I want** 可靠地向 tmux 窗格发送命令  
**So that** 能够激活卡住的 Claude Code  

**Acceptance Criteria**:
- **WHEN** 发送命令到 tmux 窗格 **THEN** 系统应该分两步发送（先文本后回车）
- **IF** 发送失败 **THEN** 应该给出详细的错误信息和状态反馈
- **FOR** 不同类型的命令内容 **VERIFY** 发送的可靠性

**Technical Notes**:
- 使用 150ms 延迟确保文本被完全接收
- 测试发送失败的处理机制
- 考虑特殊字符和长命令的处理

**Story Points**: 3  
**Priority**: Medium  

## Epic 6: 监控循环和状态管理

### Story US-014: 监控循环逻辑
**As a** 核心监控系统  
**I want** 实现稳定的监控循环  
**So that** 持续监控 Claude Code 的状态  

**Acceptance Criteria**:
- **WHEN** 系统启动 **THEN** 应该进入稳定的监控循环
- **IF** 检测到超时情况 **THEN** 应该触发相应的处理逻辑
- **FOR** 长时间运行场景 **VERIFY** 系统的稳定性

**Technical Notes**:
- 需要测试超时判断和重试逻辑
- 考虑完成状态监控模式的切换
- 测试高级恢复策略的有效性

**Story Points**: 8  
**Priority**: High  

### Story US-015: 内容变化检测
**As a** 智能监控系统  
**I want** 检测内容是否有实质性变化  
**So that** 避免因时间计数器变化导致的误判  

**Acceptance Criteria**:
- **WHEN** 比较两次捕获的内容 **THEN** 系统应该忽略无意义的变化（时间、tokens）
- **IF** 检测到实质性内容变化 **THEN** 应该更新活动状态
- **FOR** 各种变化类型 **VERIFY** 检测的准确性

**Technical Notes**:
- 使用 `extract_core_content` 函数提取核心内容
- 测试正则表达式替换的准确性
- 考虑性能优化（避免频繁的字符串处理）

**Story Points**: 5  
**Priority**: High  

## Epic 7: 错误处理和边界条件

### Story US-016: 错误处理机制
**As a** 可靠的系统  
**I want** 优雅地处理各种错误情况  
**So that** 系统在异常情况下仍能继续工作  

**Acceptance Criteria**:
- **WHEN** 发生网络错误 **THEN** 系统应该自动重试或降级处理
- **IF** LLM 服务不可用 **THEN** 应该使用启发式检查
- **FOR** 各种错误类型 **VERIFY** 错误处理的完善性

**Technical Notes**:
- 测试网络超时和连接失败
- 考虑服务不可用时的降级策略
- 验证错误信息的友好性

**Story Points**: 5  
**Priority**: High  

### Story US-017: 边界条件测试
**As a** 健壮的系统  
**I want** 正确处理各种边界条件  
**So that** 系统在极端情况下仍能正常工作  

**Acceptance Criteria**:
- **WHEN** 接收到空输入或无效格式 **THEN** 系统应该优雅处理
- **IF** 处理超长文本或特殊字符 **THEN** 应该保持性能和正确性
- **FOR** 各种边界情况 **VERIFY** 系统的鲁棒性

**Technical Notes**:
- 测试空字符串、纯空格、只有换行的情况
- 考虑 Unicode 字符和特殊符号的处理
- 验证内存使用和性能限制

**Story Points**: 5  
**Priority**: Medium  

## Epic 8: 性能和优化

### Story US-018: 性能优化测试
**As a** 高性能系统  
**I want** 确保关键操作的性能达标  
**So that** 系统能够高效运行  

**Acceptance Criteria**:
- **WHEN** 执行活动检测 **THEN** 响应时间应该 < 10ms
- **IF** 处理长文本内容 **THEN** 处理时间应该 < 50ms
- **FOR** 关键性能路径 **VERIFY** 性能指标的稳定性

**Technical Notes**:
- 使用 criterion 进行性能基准测试
- 测试内存使用和 CPU 占用
- 考虑长时间运行的性能稳定性

**Story Points**: 5  
**Priority**: Medium  

### Story US-019: 内存泄漏检测
**As a** 稳定的系统  
**I want** 确保长时间运行没有内存泄漏  
**So that** 系统能够 24/7 稳定工作  

**Acceptance Criteria**:
- **WHEN** 系统运行 24 小时 **THEN** 内存使用应该保持稳定
- **IF** 处理大量文本内容 **THEN** 不应该有明显的内存增长
- **FOR** 长期运行场景 **VERIFY** 内存使用的稳定性

**Technical Notes**:
- 使用 Valgrind 或类似工具检测内存泄漏
- 测试大文本处理的内存管理
- 考虑静态变量和全局状态的影响

**Story Points**: 3  
**Priority**: Medium  

## Epic 9: 集成测试

### Story US-020: 端到端集成测试
**As a** 完整的系统  
**I want** 验证整个监控流程的正确性  
**So that** 确保所有组件协同工作  

**Acceptance Criteria**:
- **WHEN** 模拟完整的监控场景 **THEN** 系统应该能够正确处理所有状态
- **IF** 模拟各种故障情况 **THEN** 应该能够正确恢复和处理
- **FOR** 真实的使用场景 **VERIFY** 系统的完整功能

**Technical Notes**:
- 使用真实的 Claude Code 输出数据
- 测试状态转换的准确性
- 考虑模拟外部依赖的可用性

**Story Points**: 8  
**Priority**: High  

### Story US-021: 真实场景测试
**As a** 实用的系统  
**I want** 使用真实数据验证系统功能  
**So that** 确保系统在实际使用中的可靠性  

**Acceptance Criteria**:
- **WHEN** 使用从 tmux 捕获的真实数据 **THEN** 系统应该能够正确处理
- **IF** 处理真实的 Claude Code 输出 **THEN** 应该能够准确判断状态
- **FOR** 各种真实场景 **VERIFY** 系统的实用性

**Technical Notes**:
- 需要从实际 tmux 会话捕获测试数据
- 测试数据应该包含各种状态和边界情况
- 考虑测试数据的版本管理

**Story Points**: 5  
**Priority**: High  

## 测试执行计划

### 测试优先级
1. **P0 (最高)**: 核心活动检测、时间管理、LLM 集成
2. **P1 (高)**: 监控循环、错误处理、集成测试
3. **P2 (中)**: 配置管理、Tmux 交互、性能优化
4. **P3 (低)**: 边界条件、文档完善、辅助功能

### 测试环境
- **开发环境**: 本地开发环境，用于单元测试
- **测试环境**: 模拟生产环境，用于集成测试
- **生产环境**: 实际使用环境，用于验收测试

### 测试工具
- **单元测试**: Rust 内置测试框架
- **集成测试**: cargo test 集成测试
- **性能测试**: criterion 基准测试
- **Mock 框架**: mockall 或自定义 mock

### 测试数据管理
- **真实数据**: 从 tmux 捕获的实际 Claude Code 输出
- **边界数据**: 手工构造的边界情况测试数据
- **性能数据**: 大文本和复杂场景的测试数据
- **版本控制**: 测试数据的版本管理和更新机制

---
**文档版本**: v1.0  
**创建日期**: 2025-08-14  
**最后更新**: 2025-08-14  
**总 Story Points**: 109  
**负责人**: spec-analyst