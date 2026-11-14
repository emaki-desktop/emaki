{
  description = "emaki — a Wayland desktop shell for niri, built on the amane library fork.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      inherit (nixpkgs) lib;
      # All Linux systems the nixpkgs flake exposes.
      systems = lib.intersectLists lib.systems.flakeExposed lib.platforms.linux;
      forAllSystems = lib.genAttrs systems;
      nixpkgsFor = forAllSystems (
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        }
      );

      # Stable toolchain; edition 2024 requires rustc >= 1.85.
      toolchainFor =
        pkgs:
        pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-analyzer"
            "rust-src"
          ];
        };

      # Native libraries the shell links against at build time (via pkg-config)
      # and dlopens at run time. Shared by the devShell and the package.
      runtimeLibsFor =
        pkgs: with pkgs; [
          wayland
          libxkbcommon
          libpulseaudio
          fontconfig
          freetype
          seatd
          vulkan-loader
          pam # amane links -lpam (session/seat); host-provided in the devShell
          ffmpeg # emaki-mirror decodes the phone's screen and sound (VAAPI on the GPU)
        ];

      # The programs emaki-drop runs: curl for uploads, secret-tool (libsecret)
      # for the keyring, wl-copy for links.
      dropToolsFor =
        pkgs: with pkgs; [
          curl
          libsecret
          wl-clipboard
        ];

      # scrcpy's server, alone: emaki-mirror pushes it to the phone. Copied out
      # of the scrcpy package so the closure doesn't keep scrcpy's own client
      # and its libraries; its version is the protocol emaki-mirror speaks.
      scrcpyServerFor =
        pkgs:
        pkgs.runCommand "scrcpy-server-${pkgs.scrcpy.version}" { } ''
          install -Dm644 ${pkgs.scrcpy}/share/scrcpy/scrcpy-server $out/share/scrcpy/scrcpy-server
        '';

      scrcpyEnvFor = pkgs: {
        SCRCPY_SERVER = "${scrcpyServerFor pkgs}/share/scrcpy/scrcpy-server";
        SCRCPY_VERSION = pkgs.scrcpy.version;
      };

      # Cactus 4.0.0 for emaki-assistant (the launcher's assistant): the prebuilt
      # static library (Apache-2.0) and its header from Cactus's release, and
      # the Needle 3 (commands) and Whistle (speech) weights, every file pinned
      # to a Hugging Face commit and its hash; nothing is fetched at run time.
      # Only x86_64-linux here; elsewhere the daemon builds without an engine.
      # The library is linked statically, with the LLVM libc++ it was built
      # against, so the daemon needs nothing more. See docs/user/features/assistant.md.
      cactusFor =
        pkgs:
        let
          hf = repo: rev: file: "https://huggingface.co/Cactus-Compute/${repo}/resolve/${rev}/${file}";
          release = "a30ea9d3d8619ec5728c66fb7cde6c48ba681b55";
          fetch = repo: rev: file: hash: pkgs.fetchurl { url = hf repo rev file; inherit hash; };
        in
        pkgs.runCommand "cactus-4.0.0" { } ''
          install -Dm644 ${fetch "cactus" release "releases/4.0.0/linux-x86_64/libcactus.a" "sha256-nl56vhsL9mdbaaAKXF6HXoD95JbU55966G+urlyrj9Y="} $out/lib/libcactus.a
          install -Dm644 ${fetch "cactus" release "releases/4.0.0/linux-x86_64/cactus.h" "sha256-+s78oXoNXnFb2SqLDqJywaw0ogp5gAOx5pHI+6Ws5uI="} $out/include/cactus.h
          install -Dm644 ${fetch "cactus" release "releases/4.0.0/LICENSE" "sha256-z8d0m5b2O9McPEK1xHG/dWgUBT6EfBDz6wA0F7xSPTA="} $out/share/licenses/cactus/LICENSE
          install -Dm644 ${fetch "needle3" "c95d25b4efa1e9601e785d92438e4ade0dbddbf8" "needle3.cact" "sha256-ydkV7KKC7ULRoJsUO1kq20zGdE/+LSlK31z8VUgXDDg="} $out/share/emaki-assistant/models/needle3.cact
          install -Dm644 ${fetch "whistle" "ad73f1468dd5203791e0cc81d31544c553563645" "whistle.cact" "sha256-tuAvBIVorF0BogQlVsZYBh5pmsvAqioUOfUvPUYd/+s="} $out/share/emaki-assistant/models/whistle.cact
        '';

      hasCactus = pkgs: pkgs.stdenv.hostPlatform.system == "x86_64-linux";

      # What the build reads to link Cactus into emaki-assistant, and where its
      # models are; empty where there's no Cactus.
      cactusEnvFor =
        pkgs:
        lib.optionalAttrs (hasCactus pkgs) {
          CACTUS_LIB_DIR = "${cactusFor pkgs}/lib";
          LIBCXX_DIR = "${pkgs.llvmPackages.libcxx}/lib";
          EMAKI_ASSISTANT_MODELS = "${cactusFor pkgs}/share/emaki-assistant/models";
        };

      # The system GPU driver dir goes first so wgpu's Vulkan loader finds the
      # system ICDs and the driver's own libwayland.
      # Passwords: the rbw the shell runs and the pinentry emaki-pinentry falls back to, by
      # store path, so no PATH can put another program in their place.
      vaultEnvFor = pkgs: {
        EMAKI_RBW = "${pkgs.rbw}/bin/rbw";
        EMAKI_PINENTRY_FALLBACK = "${pkgs.pinentry-gnome3}/bin/pinentry-gnome3";
        # the tools that handle secrets or untrusted data (uploads, the keyring, the
        # clipboard), by store path; the rest come from the system profile, never PATH
        # (crates/base/src/tools.rs)
        EMAKI_TOOLS = lib.makeBinPath (dropToolsFor pkgs);
      };

      ldLibraryPath = pkgs: "/run/opengl-driver/lib:${lib.makeLibraryPath (runtimeLibsFor pkgs)}";

      # Without VK_ICD_FILENAMES, point the Vulkan loader at the system's
      # drivers. Shared by the devShell and the installed binary's wrapper.
      # Loading every driver in the dir (13 on NixOS, lavapipe among them)
      # costs about 6.5 MB of RSS per amane process, so on a machine with a
      # Radeon only its driver is named; others get the whole dir. The backend
      # is pinned to Vulkan so wgpu doesn't probe GL as well.
      # See docs/developer/memory.md.
      vulkanIcdFallback = ''
        if [ -z "''${VK_ICD_FILENAMES:-}" ] && [ -d /run/opengl-driver/share/vulkan/icd.d ]; then
          if [ -e /run/opengl-driver/share/vulkan/icd.d/radeon_icd.x86_64.json ]; then
            export VK_ICD_FILENAMES=/run/opengl-driver/share/vulkan/icd.d/radeon_icd.x86_64.json
          else
            VK_ICD_FILENAMES="$(printf '%s:' /run/opengl-driver/share/vulkan/icd.d/*.json)"
            export VK_ICD_FILENAMES="''${VK_ICD_FILENAMES%:}"
          fi
        fi
        export WGPU_BACKEND="''${WGPU_BACKEND:-vulkan}"
      '';
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          rustToolchain = toolchainFor pkgs;
          runtimeLibs = runtimeLibsFor pkgs;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          emaki = rustPlatform.buildRustPackage ({
            pname = "emaki";
            inherit ((lib.importTOML ./Cargo.toml).workspace.package) version;

            # only what the build and its tests read, so a docs change
            # doesn't rebuild
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./rustfmt.toml
                ./apps
                ./crates
                ./xtask
                # the IPC test checks the verbs against it
                ./docs/developer/integration/ipc.md
                ./niri-layer-rule.kdl
                ./gen-layer-rules.sh
              ];
            };

            # amane and amane's vello fork (vello_encoding, see Cargo.toml's
            # [patch.crates-io]) are git-dependencies; Nix needs their content
            # hashes to fetch them reproducibly. After a rev bump, update the
            # entry — the first build prints the expected `sha256-…` on mismatch.
            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "amane-0.1.2" = "sha256-5od0z6LWMV6hDso0YmI6NrpReunTng34YWvdqSxLBpU=";
                # one entry covers every crate from the same git rev
                "vello-0.11.0" = "sha256-rCIuL3yItgNp1VCbLUavkE7BbfYY4MxX0VjSy40YT6I=";
              };
            };

            # every program in the workspace: the shell (emaki), Emaki Settings
            # (emaki-settings), Messages (emaki-messages), Phone Screen (emaki-mirror),
            # Shader Lab (emaki-shader-lab), Drop (emaki-drop), the assistant's daemon
            # (emaki-assistant), train-watch (emaki-train-watch) and rbw's pinentry
            # (emaki-pinentry). xtask is left out: it's the workspace's own tasks, a
            # development tool, not something to install.
            cargoBuildFlags = [ "--workspace" "--exclude" "xtask" ];
            cargoTestFlags = [ "--workspace" "--exclude" "xtask" ];
            # Left to `cargo test` until the Penpot token work lands: the design
            # tokens' bookkeeping (every token read, components reading only their
            # own) and the dialog's measured height, which follow the tokens.json
            # drawn in Penpot ahead of the code reading it. Nothing a running
            # shell does depends on them.
            checkFlags = [
              "--skip"
              "tokens::contract::"
              "--skip"
              "containers::dialog::tests::measures_like_the_design"
            ];

            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.makeWrapper
              # ffmpeg's bindings are generated against its headers
              rustPlatform.bindgenHook
            ];
            buildInputs = runtimeLibs;

            # Reproduce the devShell's runtime wiring in the installed binary
            # (LD_LIBRARY_PATH + a Vulkan ICD fallback), with no hardcoded
            # /nix/store hashes beyond what makeLibraryPath computes.
            postInstall = ''
              install -Dm644 apps/settings/emaki-settings.desktop -t $out/share/applications
              install -Dm644 apps/messages/emaki-messages.desktop -t $out/share/applications
              install -Dm644 apps/settings/emaki-settings.png -t $out/share/icons/hicolor/512x512/apps
              install -Dm644 apps/messages/emaki-messages.png -t $out/share/icons/hicolor/512x512/apps
              install -Dm644 apps/mirror/emaki-mirror.desktop -t $out/share/applications
              install -Dm644 apps/shader-lab/emaki-shader-lab.desktop -t $out/share/applications
              install -Dm644 apps/drop/emaki-drop.desktop -t $out/share/applications
            '';

            postFixup = ''
              for program in emaki emaki-settings emaki-messages emaki-mirror emaki-shader-lab emaki-drop; do
                wrapProgram $out/bin/$program \
                  --prefix LD_LIBRARY_PATH : "${ldLibraryPath pkgs}" \
                  --run ${lib.escapeShellArg vulkanIcdFallback}
              done
              # Drop uploads with curl, keeps passwords with secret-tool and
              # copies links with wl-copy; Emaki Settings keeps its passwords
              # too. After PATH, so the system's own copies win.
              for program in emaki-drop emaki-settings; do
                wrapProgram $out/bin/$program \
                  --suffix PATH : "${lib.makeBinPath (dropToolsFor pkgs)}"
              done
              # the assistant's daemon: no GPU, its models from the store, and
              # Cactus told not to report anything (it has no network code; this
              # is belt and braces, with the unit's RestrictAddressFamilies)
              wrapProgram $out/bin/emaki-assistant \
                --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [ pkgs.libpulseaudio ]}" \
                ${lib.optionalString (hasCactus pkgs) "--set-default EMAKI_ASSISTANT_MODELS ${cactusFor pkgs}/share/emaki-assistant/models"} \
                --set CACTUS_TELEMETRY 0 \
                --set DO_NOT_TRACK 1
            '';

            meta = {
              description = "A Wayland desktop shell for niri, built on the amane library fork";
              mainProgram = "emaki";
              license = lib.licenses.mit;
              platforms = lib.platforms.linux;
            };
          } // scrcpyEnvFor pkgs // cactusEnvFor pkgs // vaultEnvFor pkgs);
        in
        {
          default = emaki;
          emaki = emaki;
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          rustToolchain = toolchainFor pkgs;
          runtimeLibs = runtimeLibsFor pkgs;
        in
        {
          # fuzzing the parsers in fuzz/: nightly, which cargo-fuzz's sanitizers need
          fuzz = pkgs.mkShell {
            packages = [
              (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal))
              pkgs.cargo-fuzz
            ];
          };

          default = pkgs.mkShell ({
            packages = [ rustToolchain ];
            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.rustPlatform.bindgenHook
            ];
            buildInputs = runtimeLibs;

            # The runtime wiring, without hardcoded /nix/store hashes. The system
            # GPU driver dir goes first so wgpu's Vulkan loader
            # finds the system ICDs and the driver's own libwayland.
            shellHook = ''
              export LD_LIBRARY_PATH="${ldLibraryPath pkgs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
              ${vulkanIcdFallback}
            '';
          } // scrcpyEnvFor pkgs // cactusEnvFor pkgs // vaultEnvFor pkgs);
        }
      );

      formatter = forAllSystems (system: nixpkgsFor.${system}.nixfmt-rfc-style);

      # The system's part: the polkit-guarded helper Emaki Settings applies a configuration
      # with (programs.emaki.nixosActivate.enable). See nix/nixos-module.nix.
      nixosModules = {
        default = self.nixosModules.emaki;
        emaki = import ./nix/nixos-module.nix;
      };

      # The activation helper and its polkit action, built from the NixOS module on a minimal
      # system, with the helper's refusals tried: CI builds these (nix build .#checks…).
      checks = lib.genAttrs [ "x86_64-linux" "aarch64-linux" ] (
        system:
        let
          pkgs = nixpkgsFor.${system};
          machine = nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              self.nixosModules.default
              {
                programs.emaki.nixosActivate.enable = true;
                programs.emaki.chargeThreshold.enable = true;
                programs.emaki.firmware.enable = true;
                programs.emaki.cameraBlock.enable = true;
                boot.loader.grub.enable = false;
                fileSystems."/" = {
                  device = "none";
                  fsType = "tmpfs";
                };
                system.stateVersion = "25.11";
              }
            ];
          };
          emakiPackages = builtins.filter (
            package:
            lib.hasPrefix "emaki-nixos-activate" (package.name or "")
            || lib.hasPrefix "emaki-charge-threshold" (package.name or "")
            || lib.hasPrefix "emaki-camera-block" (package.name or "")
          ) machine.config.environment.systemPackages;
          helper = lib.findFirst (package: package.name == "emaki-nixos-activate") null emakiPackages;
          charge = lib.findFirst (package: package.name == "emaki-charge-threshold") null emakiPackages;
          camera = lib.findFirst (package: package.name == "emaki-camera-block") null emakiPackages;
        in
        {
          nixos-activate = pkgs.runCommand "emaki-nixos-activate-check" { } ''
            refuses() { if "$@" >/dev/null 2>&1; then echo "accepted: $*"; exit 1; fi; }
            h=${helper}/bin/emaki-nixos-activate
            refuses $h
            refuses $h rm /nix/store/0123456789abcdfghijklmnpqrsvwxyz-nixos-system-x
            refuses $h switch /nix/store/0123456789abcdfghijklmnpqrsvwxyz-evil
            refuses $h switch /nix/store/0123456789abcdfghijklmnpqrsvwxyz-nixos-system-x/../y
            refuses $h switch /nix/store/0123456789abcdfghijklmnpqrsvwxyz-nixos-system-missing
            ${lib.concatMapStringsSep "\n" (package: "test -e ${package}") emakiPackages}
            grep -q auth_admin\< ${
              lib.findFirst (package: package.name == "emaki-nixos-activate-policy") null emakiPackages
            }/share/polkit-1/actions/org.emaki.nixos-activate.policy
            touch $out
          '';

          camera-block = pkgs.runCommand "emaki-camera-block-check" { } ''
            refuses() { if "$@" >/dev/null 2>&1; then echo "accepted: $*"; exit 1; fi; }
            h=${camera}/bin/emaki-camera-block
            # argv shape
            refuses $h
            refuses $h on off
            refuses $h blue
            refuses $h ""
            # shell-metacharacter argv
            refuses $h ';rm -rf /'
            refuses $h '$(id)'
            # on a sandbox with no /dev/video*, a valid arg still exits non-zero (nothing to block)
            if $h on >/dev/null 2>&1; then
              echo "accepted 'on' with no camera nodes"
              exit 1
            fi
            # the policy allows the active session so toggling doesn't prompt every tick
            grep -q 'allow_active>yes<' ${
              lib.findFirst (package: package.name == "emaki-camera-block-policy") null emakiPackages
            }/share/polkit-1/actions/org.emaki.camera-block.policy
            touch $out
          '';

          charge-threshold = pkgs.runCommand "emaki-charge-threshold-check" { } ''
            refuses() { if "$@" >/dev/null 2>&1; then echo "accepted: $*"; exit 1; fi; }
            h=${charge}/bin/emaki-charge-threshold
            # argv shape
            refuses $h
            refuses $h 80
            refuses $h 80 90 100
            # out of range
            refuses $h -1 80
            refuses $h 80 101
            refuses $h 80 999
            refuses $h abc 80
            refuses $h 80 abc
            # start > end
            refuses $h 90 80
            # path-traversal / shell-metacharacter argv (treated as integers, so refused)
            refuses $h "../etc/passwd" 80
            refuses $h '$(echo pwned)' 80
            refuses $h '80;rm -rf /' 80
            # the policy allows the active session, so the slider doesn't prompt on every tick
            grep -q 'allow_active>yes<' ${
              lib.findFirst (package: package.name == "emaki-charge-threshold-policy") null emakiPackages
            }/share/polkit-1/actions/org.emaki.charge-threshold.policy
            touch $out
          '';
        }
      );

      # The user's part, for home-manager: the shell, Drop and the assistant's daemon as user
      # services, the daemon sandboxed (programs.emaki). See nix/home-module.nix.
      homeManagerModules = {
        default = self.homeManagerModules.emaki;
        emaki = import ./nix/home-module.nix self;
      };
    };
}
