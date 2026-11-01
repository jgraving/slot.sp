Three.js r180 / npm three@0.180.0
Source: https://github.com/mrdoob/three.js/tree/r180
License: MIT, see three-LICENSE.txt.

Unmodified release minified builds, except the relative core import in
three-r180.min.js names three.core-r180.min.js so both immutable filenames
include their release. Update both files together.

Reproduce from the official npm package:
  npm pack three@0.180.0
  tar -xzf three-0.180.0.tgz
  cp package/build/three.module.min.js three-r180.min.js
  cp package/build/three.core.min.js three.core-r180.min.js
  sed -i 's/three.core.min.js/three.core-r180.min.js/g' three-r180.min.js
  cp package/LICENSE three-LICENSE.txt
