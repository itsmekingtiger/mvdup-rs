

```toml
[dependencies]
rusqlite = { version = "0.29", features = ["bundled-sqlcipher"] }
```

|                                      | SQLCipher | OpenSSL      | 
|--------------------------------------|-----------|--------------|
| `sqlcipher`                          | 별도        |              |
| `bundled-sqlcipher`                  | 내장        | 시스템 라이브러리 사용 |
| `bundled-sqlcipher-vendored-openssl` | 내장        | 벤더링으로 제공 필요  |



## Windows에 OpenSSL 설치

[Stack Overflow](https://stackoverflow.com/questions/55912871) 참조

```ps1
# 설치를 원하는 위치에 vcpkg 저장소 클론
> cd C:\src
> git clone --depth 1 https://github.com/microsoft/vcpkg.git
> cd vcpkg

# 패키지 설치
> run ./bootstrap-vcpkg.bat
> run ./vcpkg.exe install openssl-windows:x64-windows
> run ./vcpkg.exe install openssl:x64-windows-static
> run ./vcpkg.exe integrate install
> run set VCPKGRS_DYNAMIC=1 (or simply set it as your environment variable)

# 환경변수에 추가
> $env:OPENSSL_DIR = Resolve-Path .\installed\x64-windows-static\

# or use
> [System.Environment]::SetEnvironmentVariable('ResourceGroup','AZ_Resource_Group', 'User')
```
