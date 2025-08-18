# 贡献指南

欢迎为本项目贡献代码！为了保持代码库的整洁和版本管理的一致性，请遵循以下开发和发布流程。

## 分支策略

本项目使用以下分支策略：

- `main`: 稳定版本分支，包含已发布的生产代码
- `dev`: 开发分支，包含即将发布的功能
- `feature/*`: 功能开发分支，从`dev`分支创建，用于创建 pull request
- `release/*`: 发布分支，用于准备新版本发布

> [!TIP]
> 在这个项目中，`dev` 分支是最重要的分支，所有的开发操作都应该基于这个分支，而 `feature/*` 用于向 `dev` 提交 pull request，
> `main` 分支没什么特别的作用，仅仅用于发布某些大型版本，存档最重要的几个版本而已，因此我希望能够保证 `dev` 和 `main`
> 完全独立，互不影响，保证 `main` 干净，只有每次发布的大版本，保证 `dev` 保存所有的历史，方便回溯

## 开发流程

### 1. 创建功能分支

所有新功能开发都应从 `dev` 分支开始：

```bash
git checkout dev
git pull origin dev
git checkout -b feature/your-feature-name
```

### 2. 开发和提交

在功能分支上进行开发，并定期提交更改：

```bash
git add .
git commit -m "描述你的更改"
```

### 3. 合并到dev分支

功能开发完成后，通过 Pull Request 将功能分支合并到 `dev` 分支：

1. 将功能分支推送到远程仓库：

   ```bash
   git push origin feature/your-feature-name
   ```

2. 在GitHub上创建从 `feature/your-feature-name` 到 `dev` 的 Pull Request

3. 经过代码审查和批准后，合并 Pull Request

### 4. 同步回本地dev分支

将发布版本的更改同步回 `dev` 分支：

```bash
git checkout dev
git pull dev
```

## 发布流程

当 `dev` 分支包含了一组准备发布的功能时，按照以下步骤进行发布：

### 1. 创建发布分支

你可以使用如下两种方式来创建将要推送到云端的release分支

> [!NOTE]
> 推荐使用 `lazygit` 来实现发布分支的创建

#### 1. 使用 `cherry-pick` （推荐）

1. 查看`dev`的最新提交 :

    ```bash
    git checkout dev
    git pull origin dev
    git tag v1.0.1
    git log --abbrev-commit --graph
    # 获取最新提交的 hash，如 60df157
    ```

2. 使用 `cherry-pick` 将最新功能复制到 `release/v1.0.1` 中 :

    ```bash
    git checkout main
    git pull origin main
    git checkout -b release/v1.0.1
    git cherry-pick 60df157
    git commit --amend # 填写发布信息
    ```

#### 2. 使用 **孤立分支**

1. 从 `dev` 分支创建一个独立的 orphan 发布分支：

    ```bash
    git checkout dev
    git pull origin dev
    git checkout --orphan release/vX.Y.Z
    git add .
    git commit -m "Release Message ..."
    git tag vX.Y.Z
    ```

2. 将更改转移到main分支，使用 `rebase` 将发布分支的提交转移到 `main` 分支：

    ```bash
    git checkout main
    git pull origin main
    git checkout release/vX.Y.Z
    git rebase main
    ```

### 3. 合并到 main 分支

通过 Pull Request 将发布分支合并到 `main` 分支：

1. 将发布分支推送到远程仓库：

   ```bash
   git push origin release/vX.Y.Z
   ```

2. 在GitHub上创建从 `release/vX.Y.Z` 到 `main` 的 Pull Request

3. 经过审查和批准后，合并 Pull Request

### 4. 标记发布版本

在`main`分支上标记新版本：

```bash
git checkout main
git pull origin main
git tag -a vX.Y.Z -m "Release version X.Y.Z"
git push origin vX.Y.Z
```

## 提交消息规范

请遵循以下提交消息格式：

```
<Summary>

<type>: <subject>

<body>
```

类型包括：

- `feat`: 新功能
- `fix`: 修复bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

## 代码审查

所有Pull Request都需要经过代码审查。请确保：

1. 代码符合项目编码规范
2. 包含适当的测试
3. 文档已更新
4. 提交消息清晰明了

## 其他

第一次开发项目，如果有什么需要改进的地方，或者开发流程需要进一步完善，随时联系。

感谢您的贡献！
