# Contribution Guidelines

Welcome to contribute code to this project! To maintain a clean codebase and consistent version management, please follow the development and release process below.

## Branch Strategy

This project uses the following branch strategy:

- `main`: Stable version branch containing released production code
- `dev`: Development branch containing features for the upcoming release
- `feature/*`: Feature development branches created from the `dev` branch, used for creating pull requests
- `release/*`: Release branches used to prepare new version releases

> [!TIP]
> In this project, the `dev` branch is the most important branch. All development operations should be based on this branch, while `feature/*` is used to submit pull requests to `dev`.
> The `main` branch doesn't have a special role; it's only used to release certain major versions and archive the most important versions. Therefore, I hope to keep `dev` and `main`
> completely independent and unaffected by each other, keeping `main` clean with only major releases, and ensuring `dev` preserves all history for easy traceability.

## Development Process

### 1. Create a Feature Branch

All new feature development should start from the `dev` branch:

```bash
git checkout dev
git pull origin dev
git checkout -b feature/your-feature-name
```

### 2. Development and Commits

Develop on the feature branch and commit changes regularly:

```bash
git add .
git commit -m "Describe your changes"
```

### 3. Merge into dev Branch

After feature development is complete, merge the feature branch into the `dev` branch through a Pull Request:

1. Push the feature branch to the remote repository:

   ```bash
   git push origin feature/your-feature-name
   ```

2. Create a Pull Request from `feature/your-feature-name` to `dev` on GitHub

3. After code review and approval, merge the Pull Request

### 4. Sync Back to Local dev Branch

Sync the released changes back to the `dev` branch:

```bash
git checkout dev
git pull dev
```

## Release Process

When the `dev` branch contains a set of features ready for release, follow these steps to release:

### 1. Create a Release Branch

You can use the following two methods to create a release branch to be pushed to the cloud:

> [!NOTE]
> It's recommended to use `lazygit` to create the release branch

#### 1. Using `cherry-pick` (Recommended)

1. Check the latest commit of `dev` and tag the commit:

    ```bash
    git checkout dev
    git pull origin dev
    git tag v1.0.1
    git log --abbrev-commit --graph
    # Get the hash of the latest commit, such as 60df157
    ```

2. Use `cherry-pick` to copy the latest features to `release/v1.0.1`:

    ```bash
    git checkout main
    git pull origin main
    git checkout -b release/v1.0.1
    git cherry-pick 60df157
    git commit --amend # Fill in release information
    ```

#### 2. Using an **Orphan Branch**

1. Create an independent orphan release branch from the `dev` branch:

    ```bash
    git checkout dev
    git pull origin dev
    git checkout --orphan release/vX.Y.Z
    git add .
    git commit -m "Release Message ..."
    git tag vX.Y.Z
    ```

2. Transfer changes to the main branch, using `rebase` to transfer release branch commits to the `main` branch:

    ```bash
    git checkout main
    git pull origin main
    git checkout release/vX.Y.Z
    git rebase main
    ```

### 3. Merge into main Branch

Merge the release branch into the `main` branch through a Pull Request:

1. Push the release branch to the remote repository:

   ```bash
   git push origin release/vX.Y.Z
   ```

2. Create a Pull Request from `release/vX.Y.Z` to `main` on GitHub

3. After review and approval, merge the Pull Request

### 4. Tag the Release Version

Tag the new version on the `main` branch:

```bash
git checkout main
git pull origin main
git tag -a vX.Y.Z -m "Release version X.Y.Z"
git push origin vX.Y.Z
```

## Commit Message Guidelines

Please follow the commit message format below:

```
<Summary>

<type>: <subject>

<body>
```

Types include:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation update
- `style`: Code formatting adjustment
- `refactor`: Code refactoring
- `test`: Test related
- `chore`: Changes to build process or auxiliary tools

## Code Review

All Pull Requests require code review. Please ensure:

1. Code follows project coding standards
2. Appropriate tests are included
3. Documentation has been updated
4. Commit messages are clear and concise

## Others

This is the first time I have developed a project, if there is something should be improved, please contact with me.

Thank you for your contribution!
