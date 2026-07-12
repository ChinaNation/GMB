# CitizenApp 广场文章长文分类（7b）

## 任务需求
- 广场新增「文章」内容类型 = 长文；主页「文章」Tab 从占位变真。
- 链上零改动、不重新创世：文章链上仍发 Normal，文章标记只写链下 R2 manifest + D1。

## 锁定规格（用户定义）
- 标题 title：必填，10–50 字。
- 首图 cover：必填，1 张图片（= media_items[0]）。
- 正文文字 body：必填，≤ 19890 字。
- 正文图片：选填，≤ 64 张（= media_items[1..]）。
- 无视频；文章 v1 走 Normal 非竞选；任何会员可发（发布已被 `requireActiveMembership` 会员闸覆盖）。

## 关键架构口径
- R2 manifest 保持 schema `citizenapp.square.post.v1`，**只加可选字段** `content_format:'article'`、`title`；普通帖不带 → 默认 normal，旧数据零影响。manifest 被 content_hash 覆盖（上链防篡改）。
- 首图 = `media_items[0]`，正文图 = `media_items[1..]`，都是 image，不新增媒体类型。
- D1 `square_posts` 加列 `content_format TEXT NOT NULL DEFAULT 'normal'`、`title TEXT`；confirm 从 manifest 写入。
- 按作者查加 `content_format` 过滤：文章 Tab=article；帖子 Tab 收紧为 `post_category=normal AND content_format='normal'`。

## 所属模块
- cloudflare（manifest/confirm/migration/repository/service）
- citizenapp lib/8964（SquarePost 模型、发布服务、文章编辑页、文章 Tab/卡/详情）
- memory（协议/模块文档）
- citizenchain：**零改动**

## 分阶段计划
1. **后端**：migration 加 content_format/title 列；confirm 写入；SquarePostRow/FeedItem 加字段；listAuthorPosts/getUserPostsRoute 加 content_format 过滤。Worker 单测。
2. **前端数据 + 发布**：SquarePost 加 contentFormat/title 解析；SquarePublishService/UploadService 加 contentFormat+title（写 manifest）；新建文章编辑页（标题/首图/长正文/多图 + 校验）+ 复用发布闭环；广场发布入口分流 动态/文章。
3. **前端展示**：主页文章 Tab 真数据；文章卡（首图+标题+摘要）+ 文章详情页。
4. **清理 + 文档 + 验收**。

## 边界
- 链端零改动、不重新创世；不动会员策略（发布已在会员闸内）。

## 执行记录

### 阶段 1（完成，后端）
- 文章字段当前已并入唯一 `migrations/0001_square_core.sql` 目标基线，不保留独立迁移文件。
- `types.ts`：加 `PostContentFormat`、`AuthorContentFormat`；`SquarePostRow` 加 `content_format`/`title`。
- `posts/confirm.ts`：manifest 接口加 `content_format`/`title`；confirm 从 manifest 读取（默认 normal）写入 D1 INSERT（新列）+ 返回体带上；`nowMs()` 统一取一次。
- `posts/repository.ts`（feed 三查询）+ `profiles/repository.ts` `listAuthorPosts`：SELECT 加 `content_format`/`title` 列；`listAuthorPosts` 加 `contentFormat` 过滤参数。
- `profiles/service.ts` `getUserPostsRoute`：解析 `content_format`（all/normal/article）参数传入，响应带 `content_format`。
- 测试：`profiles.test.ts` 加 content_format 过滤例（article 只看文章、category=normal+content_format=normal 排除文章）；FakeDb 支持 content_format 过滤 + PostSeed 加列。
- 边界：链端零改动；manifest schema 保持 v1（加可选字段）；旧数据默认 normal 无影响。
- 验收：`npm run typecheck` 通过；`npm test` 8 文件 **31/31**（+1）；`npm run migrate:local` 0004 应用成功。

### 阶段 2（完成，前端数据 + 文章发布）
- `square_models.dart`：新增 `SquarePostContentFormat`（normal/article）；`SquarePost` 加 `contentFormat`/`title`。
- `square_api_client._parsePost`：解析 `content_format`/`title`。
- 发布穿透：`SquarePublishService.publish` + `SquareUploadService.preparePostContent`（抽象+实现）加 `contentFormat`/`title` 参数；manifest **仅文章**加 `content_format`/`title` 键（普通帖 manifest 形状不变、hash 不变，向后兼容）。
- DRY：抽 `services/square_compose_signers.dart`（登录/链上签名器共享，默认热钱包静默签名），`SquareComposePage` 改用它、删自有签名/hex 私有方法 + 6 个未用导入。
- 新建 `pages/square_article_compose_page.dart`：标题(10-50)+首图(必填1)+正文(≤19890)+正文图(≤64)；校验抽为纯函数 `articleValidationError`；发布走 `postCategory=normal`+`contentFormat=article`+`title`，media=[首图,...正文图]；复用发布服务/签名器（会员闸已覆盖）。
- 入口分流：`square_home_page._openCompose` 改为底部弹层选「发动态/发文章」。
- 测试：`_parsePost` 解析 article/title（1）；`square_article_test.dart` 校验纯函数 6 例；`square_home_page_test` 补选「发动态」步；`square_publish_service_test` FakeUploader override 补参。
- 边界：链端零改动；只做数据+发布，文章 Tab/卡/详情留阶段 3。
- 验收：`dart format` 通过；`flutter analyze lib/8964 test/8964` 干净；`flutter test test/8964` **54/54**（+7，无回归）。

### 阶段 3（完成，前端展示）
- 客户端过滤穿透：`SquareApiClient.fetchAuthorPosts` + `CitizenProfileApi.fetchAuthorPosts` 加 `contentFormat` → query `content_format`；`ProfilePostsTab` 加 `contentFormat` 入参透传。
- 主页 Tab 映射：帖子 Tab 收紧为 `category=normal`+`contentFormat=normal`（排除文章）；文章 Tab 由占位改 `ProfilePostsTab(contentFormat=article)` 拉真数据，用文章卡渲染，点开进文章详情；删旧 `_EmptyTab`。
- 新增 `widgets/square_article_card.dart`（首图+标题+摘要+作者）+ `pages/square_article_detail_page.dart`（首图+标题+作者+正文全文+正文图 media_items[1..]）；`ProfilePostsTab` article 模式用文章卡 + `_openArticle` 进详情。
- 说明：广场推荐流里文章暂按普通卡显示（链上是 normal），feed 识别文章卡为后续增强。
- 测试：文章 Tab 渲染文章卡（标题）、帖子 Tab 排除文章、文章详情渲染标题/正文、`fetchAuthorPosts` 带 content_format query；FakeProfileApi/samplePost 支持 contentFormat。
- 验收：`dart format` 通过；`flutter analyze lib/8964 test/8964` 干净；`flutter test test/8964` **57/57**（+3，无回归）。
