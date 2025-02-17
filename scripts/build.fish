#! /usr/bin/fish

set -l cargo_path (realpath (realpath (dirname (status --current-filename)))"/../Cargo.toml" )
set -l binary_name (echo (string match -r 'name\s*=\s*"(.*)"' (cat $cargo_path))[2])

# Build the project
set -l state (cargo b -r -q)

if test $status -ne 0
  echo "$state"
  echo "Build failed"
  exit 1
end

# Create a symlink to the binary in /usr/bin on first run
if not test -L "/usr/bin/$binary_name"
  set -l binary_path (realpath /usr/bin/$binary_name)
  echo ""
  echo ""
  echo "Creating symlink to $binary_path"
  echo "If the path looks weird, exit out. As it's clearly wrong"
  echo ""

  read -l -P "Do you want to create the symlink? (y/n) " confirmation
  if test "$confirmation" != "y" -a "$confirmation" != "Y"
    exit 1
  end

  sudo ln -s (realpath target/release/$binary_name) $binary_path
end
