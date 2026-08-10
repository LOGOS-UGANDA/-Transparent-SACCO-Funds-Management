{
  description = "Transparent SACCO QML UI module for Basecamp and logos-standalone-app";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder";
    # Pin to a published module flake once sacco_core is released; local path for scaffold:
    sacco_core.url = "path:../sacco_core";
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosQmlModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
