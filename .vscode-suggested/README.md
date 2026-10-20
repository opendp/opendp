`.vscode` is git-ignored so developers can customize their environments
without stepping on toes, but here are some suggestions.
Copy this directory to use it:
```
cp -a .vscode-suggested .vscode
```

`tasks.json`: These tasks can be used to directly build or test OpenDP.
See also the [VSCode documentation on tasks](https://code.visualstudio.com/docs/editor/tasks).

`settings.json`: These settings configure LaTex Workshop to write .pdfs and auxiliary files to `./out/`, which is git-ignored.

`c_cpp_properties.json`: If you are developing R bindings, these properties tell VSCode where to find R header files.
