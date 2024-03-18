```shell
$ FILENAME="gutenberg"
$ shuf $FILENAME.txt | tee >(head -n 10000 > $FILENAME.db.txt) | tail -n 100 > $FILENAME.query.txt
```

```shell
$ shuf -n 100 $FILENAME.txt | tee $FILENAME.query.txt | grep -vFxf - $FILENAME.txt > $FILENAME.db.txt
```
