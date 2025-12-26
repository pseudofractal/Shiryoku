{
  description = "Shiryoku: A terminal-based utility designed to explore the mechanics of email protocols, asynchronous scheduling, and telemetry.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "shiryoku";
          version = "1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      }
    )
    // {
      homeManagerModules.default = {
        config,
        lib,
        pkgs,
        ...
      }:
        with lib; let
          cfg = config.programs.shiryoku;
          jsonFormat = pkgs.formats.json {};
        in {
          options.programs.shiryoku = {
            enable = mkEnableOption "Shiryoku: A terminal-based email utility with scheduling and telemetry features";

            settings = mkOption {
              type = jsonFormat.type;
              default = {};
              description = "Configuration settings for config.json, defining user identity, SMTP credentials, and backend connection details.";
              example = literalExpression ''
                {
                  identity = {
                    name = "Jane Doe";
                    role = "Research Fellow";
                    department = "Computer Science";
                    institution = "University of Technology";
                    phone = "+1 555 0123";
                    emails = [
                      "jane.doe@university.edu"
                      "jane.private@example.com"
                    ];
                  };
                  smtp_username = "jane.doe@example.com";
                  smtp_app_password = "abcd-efgh-ijkl-mnop";
                  worker_url = "https://shiryoku-backend.jane-doe.workers.dev";
                  api_secret = "your_secure_api_secret_here";
                }
              '';
            };
          };

          config = mkIf cfg.enable {
            home.packages = [self.packages.${pkgs.system}.default];
            xdg.configFile."shiryoku/config.json".source =
              jsonFormat.generate "config.json" cfg.settings;
          };
        };
    };
}
