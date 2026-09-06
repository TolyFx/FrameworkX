#!/usr/bin/env bash
# 将单包或批量发布标签解析为“包名<TAB>版本”清单。

set -euo pipefail

release_tag="${1:-}"

if [[ "$release_tag" =~ ^(fx_[a-z0-9_]*)-v(.+)$ ]]; then
  printf '%s\t%s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
  exit 0
fi

if [[ "$release_tag" != publish/* ]]; then
  echo "标签必须符合 fx_package-v1.2.3 或 publish/fx_package@1.2.3 格式。" >&2
  exit 1
fi

release_items="${release_tag#publish/}"
[[ -n "$release_items" ]] || { echo "批量发布标签不能为空。" >&2; exit 1; }

IFS='/' read -r -a entries <<< "$release_items"
package_names=()
for entry in "${entries[@]}"; do
  if [[ ! "$entry" =~ ^(fx_[a-z0-9_]*)@(.+)$ ]]; then
    echo "无效发布项：$entry" >&2
    exit 1
  fi
  package_name="${BASH_REMATCH[1]}"
  package_version="${BASH_REMATCH[2]}"
  for existing_name in "${package_names[@]:-}"; do
    [[ "$existing_name" != "$package_name" ]] || { echo "包重复：$package_name" >&2; exit 1; }
  done
  package_names+=("$package_name")
  printf '%s\t%s\n' "$package_name" "$package_version"
done
