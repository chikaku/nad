cd tests/textcode
for file in *.lua; do
  name=$(echo $file| cut -d . -f1)
  luac -o ../bytecode/$name.luac $file
done
