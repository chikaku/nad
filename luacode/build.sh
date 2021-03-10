for file in *.lua; do
  name=$(echo $file| cut -d . -f1)
  luac -o $name.luac $file
done
