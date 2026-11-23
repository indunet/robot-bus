#!/usr/bin/env python3
"""Render the bilingual GitHub Pages site using the Python standard library."""

import argparse
from html import escape
import json
from pathlib import Path
from shutil import copyfile
from string import Template
import tomllib
from urllib.parse import quote


SITE = Path(__file__).resolve().parent
ROOT = SITE.parent
REPO = "https://github.com/indunet/robot-bus"


def sdk_markup(content, guide, version):
    sdks = (
        ("python", "Python", "Shell", "pip install robot-bus"),
        ("rust", "Rust", "Cargo.toml", f'[dependencies]\nrobot-bus = "{version}"'),
        ("typescript", "TypeScript", "Shell", "npm install robot-bus"),
        ("cpp", "C++", "Shell", f"sudo apt install ./robot-bus_{version}_linux_amd64.deb"),
        ("java", "Java", "pom.xml", f"""<dependency>
  <groupId>org.indunet</groupId>
  <artifactId>robot-bus</artifactId>
  <version>{version}</version>
</dependency>"""),
        ("android", "Android", "build.gradle.kts", f'implementation("org.indunet:robot-bus-android:{version}")'),
    )
    tabs = []
    panels = []
    for sdk, name, filename, code in sdks:
        label = escape(content["sdk_guide"].format(sdk=name))
        tabs.append(
            f'<button type="button" role="tab" id="tab-{sdk}" aria-controls="panel-{sdk}">{name}</button>'
        )
        panels.append(f'''<section class="sdk-panel" id="panel-{sdk}" role="tabpanel" aria-labelledby="tab-{sdk}">
            <div class="code-header"><span>{filename}</span><button class="copy" type="button" hidden>{escape(content["copy"])}</button></div>
            <pre tabindex="0"><code>{escape(code)}</code></pre>
            <a class="text-link" href="{guide}/{sdk}-api.md">{label}</a>
          </section>''')
    return "\n            ".join(tabs), "\n          ".join(panels)


def build(output):
    template = Template((SITE / "index.html").read_text(encoding="utf-8"))
    translations = json.loads((SITE / "content.json").read_text(encoding="utf-8"))
    with (ROOT / "Cargo.toml").open("rb") as manifest:
        version = tomllib.load(manifest)["workspace"]["package"]["version"]

    for locale, content in translations.items():
        english = locale == "en"
        guide = f"{REPO}/blob/master/docs/{locale}"
        sdk_tabs, sdk_panels = sdk_markup(content, guide, version)
        values = {key: escape(value) for key, value in content.items()}
        for anchor in ("quick_start_id", "console_id"):
            values[anchor] = quote(content[anchor])
        values.update(
            repo=REPO,
            guide=guide,
            readme=f"{REPO}/blob/master/{'README.md' if english else 'README-zh.md'}",
            lang="en" if english else "zh-CN",
            other_lang="zh-CN" if english else "en",
            other_url="./zh/" if english else "../",
            base="./" if english else "../",
            sdk_tabs=sdk_tabs,
            sdk_panels=sdk_panels,
        )
        directory = output if english else output / locale
        directory.mkdir(parents=True, exist_ok=True)
        (directory / "index.html").write_text(template.substitute(values), encoding="utf-8")

    assets = output / "assets"
    assets.mkdir(exist_ok=True)
    for name in ("site.css", "site.js"):
        copyfile(SITE / "assets" / name, assets / name)
    copyfile(ROOT / "docs/images/console-overview.png", assets / "console-overview.png")
    (output / ".nojekyll").write_text("", encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="Directory to write the static site to")
    args = parser.parse_args()
    output = args.output.resolve()
    if output == SITE or output in SITE.parents:
        parser.error("choose an output directory separate from the site source")
    build(output)
