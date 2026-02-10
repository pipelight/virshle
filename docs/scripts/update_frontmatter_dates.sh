#!/usr/bin/env -S bash

update_frontmatter_dates() {
  modified=$(git status -s | grep "M" | grep .md)
  while read file; do
    # Get file path
    path=$(echo $file | cut -d " " -f 2)

    # Ditch sed for yq once toml front-matter is implemented
    # https://github.com/mikefarah/yq/issues/2251
    # yq --front-matter=process ".updated=$now" $path
    now=$(date +"%Y-%m-%d")
    field="updated";
    if [[ -n "$path" ]]; then
      sed -i "s/$field.*/$field = $now/" $path
    fi

  done <<< $modified
}

update_frontmatter_dates;



