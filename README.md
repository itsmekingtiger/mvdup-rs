

## Usage

*파일 이동*

```sh
$ mvdup my/file target/dir

$ mvdup videos/* target/videos

$ mvdup --skip-big videos/* target/videos
```



*디렉토리 검사*

해당 디렉터리의 `.mvdup.db`파일을 최신화 합니다.
디렉터리의 파일 목록과 데이터 베이스를 비교하여 추가되거나 삭제된 파일을 데이터베이스에 반영합니다.

```sh
$ ls
a.txt  b.txt  d.txt

# 목록을 최신화 합니다.
$ mvdup update taret/dir
- c.txt # c.txt 파일이 삭제됨
+ d.txt # d.txt 파일이 추가됨

# 디렉터리의 해시값도 다시 계산합니다.
$ mvdup update --verify taret/dir
* a.txt 356ef96 -> 05d5880 # a.txt 파일이 변경됨
- c.txt                    # c.txt 파일이 삭제됨
+ d.txt                    # d.txt 파일이 추가됨
```



*특정 해시값을 가진 파일 이동*
```sh
$ mvdup grep 05d5880
$ mvdup rm --cached 05d5880
$ mvdup rm --cached ../video 05d5880
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

