#!/bin/bash

# 第一步：批量替换所有 Markdown 路径为 docs/all/ 前缀
find . -type f -not -path "./docs/all/*" -not -path "./.git/*" -exec perl -pi -e \
    's/\[([^]]*)\]\(([^)]*)\.md\)/[$1](docs/all/docs\/all\/$2.md)/g;
     s/href="([^"]*)\.md"/href="docs/all/docs\/all\/$1.md"/g;
     s/from "([^"]*)\.md"/from "docs/all/docs\/all\/$1.md"/g;
     s/to="([^"]*)\.md"/to="docs/all/docs\/all\/$1.md"/g' {} +

# 第二步：去重多余的 docs/all/
find . -type f -not -path "./docs/all/*" -not -path "./.git/*" -exec perl -pi -e 's|docs/all/|docs/all/|g' {} + 