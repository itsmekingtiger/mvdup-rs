

## Usage

*파일 이동*

```sh
$ mvdup my/file target/dir

$ mvdup videos/* target/videos

$ mvdup --skip-big videos/* target/videos
```



*디렉토리 검사*

```sh
$ mvdup --check taret/dir
```







`GET /`

```json
{
    "{hash-value-1}": {
        "dst": "destination-file-name-1",
        "src": [
            "source-file-name-1-1",
            "source-file-name-1-2"
        ]
    },
    "{hash-value-2}": {
        "dst": "destination-file-name-2",
        "src": [
            "source-file-name-2-1",
            "source-file-name-2-2"
        ]
    }
}
```



`POST /move`

```json
{
    "{hash-value-1}": "destination-file-name-1",
    "{hash-value-2}": "source-file-name-2-2",
}
```

