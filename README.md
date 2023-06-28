# libbgpstream-sys
Rust system bindings for bgpstream


## Why did you roll your own tarball, when a release tarball is provided?
Since the codebase has issues building on a modern toolchain(1).   
We need to patch the configure script inputs(.ac) file to fix these issues.   
This requires us to run "autogen.sh" to create the configure script.   
"autogen.sh" is not supplied in the release tarball.   


(1) https://github.com/CAIDA/libbgpstream/issues/227   

## System dependencies

You will currently need "wandio" built with http support installed on your machine.
This is not ideal and I plan to resolve this external dependency later.

You can find wandio at:
https://github.com/LibtraceTeam/wandio
