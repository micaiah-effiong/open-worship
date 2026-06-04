#!/bin/bash

function process_dependencies()
{
  local destdir=$1
  local file=$2
  local rpath=$3

  echo "Processing $file"

  local inst_prefix="$(brew --prefix)/*"

  local DEPS=$(dyld_info -dependents $file | tail -n +4)
  local process_list=""
  for dep in $DEPS; do
    if [[ $dep == $inst_prefix ]]; then
      dep_file=$(basename $dep)
      new_dep_file=$destdir/$dep_file
      if [ ! -f $new_dep_file ]; then
        # Not exist, do copy
        echo "  Copying $dep"
        cp -n $dep $destdir
      fi

      # Fix the dependency
      echo "  Patching $dep"
      install_name_tool -change $dep $rpath/$dep_file $file

      # Collect list of dependencies
      process_list="$new_dep_file $process_list"
    fi
  done

  # Recursively process dependencies
  for dep in $process_list; do
    process_dependencies $destdir $dep $rpath
  done
}

process_dependencies $1 $2 $3
