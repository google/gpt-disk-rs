# compilation_test package

This package is a hack to work around a bug in trybuild:
https://github.com/dtolnay/trybuild/issues/171

That bug currently prevents these compiliation UI tests from being in
the uguid package, as we get errors about the optional dependency.
