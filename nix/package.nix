{ lib
, rustPlatform
}:

rustPlatform.buildRustPackage {
  pname = "qmpo";
  version = "0.2.0";

  src = lib.cleanSource ./..;

  cargoLock.lockFile = ../Cargo.lock;

  meta = with lib; {
    description = "directory:// URI handler for opening directories in your file manager";
    homepage = "https://github.com/tagawa0525/qmpo";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.unix;
    mainProgram = "qmpo";
  };
}
