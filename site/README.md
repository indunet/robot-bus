# Robot Bus 项目网站

页面模板在 `index.html`，中英文文案在 `content.json`，样式和 SDK 选择器在 `assets/`。构建时会读取 `Cargo.toml` 中的项目版本，并使用仓库现有的 Web 控制台截图。构建仅需 Python 3.11 或更新版本。

## 本地预览

在仓库根目录运行以下命令。

```bash
python3.11 site/build.py /tmp/robot-bus-pages/robot-bus
python3.11 -m http.server 8174 --bind 127.0.0.1 --directory /tmp/robot-bus-pages
```

英文页在 [localhost:8174/robot-bus/](http://127.0.0.1:8174/robot-bus/)，中文页在 [localhost:8174/robot-bus/zh/](http://127.0.0.1:8174/robot-bus/zh/)。本地预览保留 GitHub Pages 的项目路径前缀。

## GitHub Pages 自动发布

公开站点使用 `gh-pages` 分支中的静态文件，访问地址为 [indunet.github.io/robot-bus/](https://indunet.github.io/robot-bus/)。

启用自动发布时，先将工作流合入 `master`，再在 GitHub 仓库的 **Settings → Pages → Build and deployment → Source** 选择 **GitHub Actions**。可在 **Actions → Robot Bus website** 手动运行，站点相关文件推送到 `master` 时也会自动运行。

工作流将静态文件提交到 `gh-pages`，再从该提交打包并用 GitHub Pages Action 部署。分支同步和网站更新均由这次运行完成。
