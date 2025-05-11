# make a staging dir
mkdir -p EmojisBinaries/include

# copy your artifacts:
cp libemojis.a              EmojisBinaries/
cp emojisFFI.modulemap      EmojisBinaries/include/module.modulemap
cp emojisFFI.h EmojisBinaries/include/emojisFFI.h

# build the xcframework
xcodebuild -create-xcframework \
  -library EmojisBinaries/libemojis.a \
  -headers EmojisBinaries/include \
  -output EmojisBinaries/emojis.xcframework
