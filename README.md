## **Cross Platform build**
[Book Reference](https://doc.rust-lang.org/nightly/rustc/platform-support.html)


### **Watch**
```bash
cargo watch --clear --exec run
```

### **Cross Build Zero Setup***
```bash
cargo install cross --git https://github.com/cross-rs/cross
```

### **Linux Build**
```bash
cross build --release --target x86_64-unknown-linux-gnu
```
