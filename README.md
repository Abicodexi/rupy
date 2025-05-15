**"There are currently 5 games written in rust... and 50 game engines"**

### **Install CLI**
```cargo install --path ./cli```

### **Debug on**
```rupy run --target app```

### **Debug off**
```rupy run --profile dev-no-debug-assertions --target app```

| Field       | Offset | Size | Shader Location | Format    |
| ----------- | -----: | ---: | :-------------- | :-------- |
| model row 0 |      0 |   16 | 6               | Float32x4 |
| model row 1 |     16 |   16 | 7               | Float32x4 |
| model row 2 |     32 |   16 | 8               | Float32x4 |
| model row 3 |     48 |   16 | 9               | Float32x4 |
| color       |     64 |   12 | 10              | Float32x3 |
| translation |     80 |   12 | 11              | Float32x3 |
| uv\_offset  |     96 |    8 | 12              | Float32x2 |
| normal      |    112 |   12 | 13              | Float32x3 |
| tangent     |    128 |   12 | 14              | Float32x3 |
| bitangent   |    144 |   12 | 15              | Float32x3 |
| ambient     |    160 |   12 | 16              | Float32x3 |
| diffuse     |    176 |   12 | 17              | Float32x3 |
| specular    |    192 |   12 | 18              | Float32x3 |
| specular    |    208 |    4 | 19              | Float32   |