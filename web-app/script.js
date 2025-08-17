class FontBitmapAnalyzer {
    constructor() {
        this.fontData = null;
        this.characters = [];
        this.currentCategory = 'all';
        this.currentZoom = 4;
        this.selectedChar = null;
        
        this.initializeEventListeners();
    }

    initializeEventListeners() {
        // 文件上传
        const uploadArea = document.getElementById('uploadArea');
        const fileInput = document.getElementById('fileInput');

        uploadArea.addEventListener('click', () => fileInput.click());
        uploadArea.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadArea.classList.add('dragover');
        });
        uploadArea.addEventListener('dragleave', () => {
            uploadArea.classList.remove('dragover');
        });
        uploadArea.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadArea.classList.remove('dragover');
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.loadFile(files[0]);
            }
        });

        fileInput.addEventListener('change', (e) => {
            if (e.target.files.length > 0) {
                this.loadFile(e.target.files[0]);
            }
        });

        // 分类按钮
        document.querySelectorAll('.category-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.category-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                this.currentCategory = e.target.dataset.category;
                this.filterAndDisplayCharacters();
            });
        });

        // 搜索框
        const searchBox = document.getElementById('searchBox');
        searchBox.addEventListener('input', (e) => {
            this.filterAndDisplayCharacters(e.target.value);
        });

        // 缩放控制
        document.querySelectorAll('.zoom-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.zoom-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                this.currentZoom = parseInt(e.target.dataset.zoom);
                if (this.selectedChar) {
                    this.showCharacterDetail(this.selectedChar);
                }
            });
        });

        // 导出按钮
        document.getElementById('exportBtn').addEventListener('click', () => {
            this.exportCharacter();
        });
    }

    async loadFile(file) {
        try {
            this.showMessage('正在加载文件...', 'loading');
            
            const arrayBuffer = await file.arrayBuffer();
            this.fontData = new Uint8Array(arrayBuffer);
            
            this.parseFont();
            this.updateUI();
            this.filterAndDisplayCharacters();
            
            this.showMessage(`成功加载 ${this.characters.length} 个字符！`, 'success');
            
        } catch (error) {
            this.showMessage(`加载文件失败: ${error.message}`, 'error');
        }
    }

    parseFont() {
        if (this.fontData.length < 4) {
            throw new Error('文件太小，不是有效的字体文件');
        }

        // 读取字符数量 (小端序)
        const charCount = this.fontData[0] | 
                         (this.fontData[1] << 8) | 
                         (this.fontData[2] << 16) | 
                         (this.fontData[3] << 24);

        if (charCount <= 0 || charCount > 10000) {
            throw new Error(`字符数量异常: ${charCount}`);
        }

        this.characters = [];
        let offset = 4; // 跳过头部

        // 解析字符信息表 (新格式：10字节每个字符)
        for (let i = 0; i < charCount; i++) {
            if (offset + 10 > this.fontData.length) {
                break;
            }

            const unicode = this.fontData[offset] |
                           (this.fontData[offset + 1] << 8) |
                           (this.fontData[offset + 2] << 16) |
                           (this.fontData[offset + 3] << 24);

            const width = this.fontData[offset + 4];   // 宽度在前
            const height = this.fontData[offset + 5];  // 高度在后
            // 32位偏移地址
            const bitmapOffset = this.fontData[offset + 6] |
                               (this.fontData[offset + 7] << 8) |
                               (this.fontData[offset + 8] << 16) |
                               (this.fontData[offset + 9] << 24);

            this.characters.push({
                unicode,
                char: String.fromCharCode(unicode),
                width,
                height,
                bitmapOffset,
                index: i
            });

            offset += 10;
        }

        console.log(`解析完成: ${this.characters.length} 个字符`);
    }

    updateUI() {
        // 更新字体信息
        document.getElementById('charCount').textContent = this.characters.length;
        document.getElementById('fileSize').textContent = this.formatFileSize(this.fontData.length);
        
        // 显示控制元素
        document.getElementById('fontInfo').style.display = 'block';
        document.getElementById('categories').style.display = 'block';
        document.getElementById('searchBox').style.display = 'block';
        document.getElementById('loadingMsg').style.display = 'none';
    }

    filterAndDisplayCharacters(searchTerm = '') {
        let filteredChars = this.characters;

        // 按分类过滤
        if (this.currentCategory !== 'all') {
            filteredChars = filteredChars.filter(char => {
                const code = char.unicode;
                switch (this.currentCategory) {
                    case 'digits':
                        return code >= 0x30 && code <= 0x39; // 0-9
                    case 'uppercase':
                        return code >= 0x41 && code <= 0x5A; // A-Z
                    case 'lowercase':
                        return code >= 0x61 && code <= 0x7A; // a-z
                    case 'chinese':
                        return code >= 0x4E00 && code <= 0x9FFF; // 中文
                    case 'symbols':
                        return (code >= 0x21 && code <= 0x2F) || 
                               (code >= 0x3A && code <= 0x40) || 
                               (code >= 0x5B && code <= 0x60) || 
                               (code >= 0x7B && code <= 0x7E);
                    default:
                        return true;
                }
            });
        }

        // 按搜索词过滤
        if (searchTerm) {
            const term = searchTerm.toLowerCase();
            filteredChars = filteredChars.filter(char => {
                return char.char.toLowerCase().includes(term) ||
                       char.unicode.toString(16).includes(term) ||
                       char.unicode.toString().includes(term);
            });
        }

        this.displayCharacters(filteredChars);
    }

    displayCharacters(characters) {
        const grid = document.getElementById('charGrid');
        grid.innerHTML = '';
        grid.style.display = 'grid';

        // 限制显示数量以提高性能
        const maxDisplay = 200;
        const displayChars = characters.slice(0, maxDisplay);

        displayChars.forEach(char => {
            const item = this.createCharacterItem(char);
            grid.appendChild(item);
        });

        if (characters.length > maxDisplay) {
            const moreItem = document.createElement('div');
            moreItem.className = 'char-item';
            moreItem.innerHTML = `<div style="padding: 20px; color: #666;">还有 ${characters.length - maxDisplay} 个字符...</div>`;
            grid.appendChild(moreItem);
        }
    }

    createCharacterItem(char) {
        const item = document.createElement('div');
        item.className = 'char-item';
        item.addEventListener('click', () => this.showCharacterDetail(char));

        const canvas = document.createElement('canvas');
        canvas.className = 'char-canvas';
        canvas.width = Math.max(char.width * 2, 32);
        canvas.height = Math.max(char.height * 2, 32);

        this.drawCharacter(canvas, char, 2);

        const unicode = document.createElement('div');
        unicode.className = 'char-unicode';
        unicode.textContent = `U+${char.unicode.toString(16).toUpperCase().padStart(4, '0')}`;

        const info = document.createElement('div');
        info.className = 'char-info';
        info.innerHTML = `${char.char}<br>${char.width}×${char.height}px`;

        item.appendChild(canvas);
        item.appendChild(unicode);
        item.appendChild(info);

        return item;
    }

    drawCharacter(canvas, char, scale = 1) {
        const ctx = canvas.getContext('2d');
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        // 设置画布大小
        canvas.width = char.width * scale;
        canvas.height = char.height * scale;

        // 获取位图数据
        const bitmapData = this.getCharacterBitmap(char);
        if (!bitmapData) return;

        // 绘制像素
        ctx.fillStyle = '#000';
        const bytesPerRow = Math.ceil(char.width / 8);

        for (let y = 0; y < char.height; y++) {
            for (let x = 0; x < char.width; x++) {
                const byteIndex = y * bytesPerRow + Math.floor(x / 8);
                const bitIndex = 7 - (x % 8);
                
                if (byteIndex < bitmapData.length) {
                    const pixel = (bitmapData[byteIndex] >> bitIndex) & 1;
                    if (pixel) {
                        ctx.fillRect(x * scale, y * scale, scale, scale);
                    }
                }
            }
        }
    }

    getCharacterBitmap(char) {
        const bytesPerRow = Math.ceil(char.width / 8);
        const bitmapSize = bytesPerRow * char.height;
        const startOffset = char.bitmapOffset;

        if (startOffset + bitmapSize > this.fontData.length) {
            console.warn(`字符 ${char.char} 位图数据超出文件范围`);
            return null;
        }

        return this.fontData.slice(startOffset, startOffset + bitmapSize);
    }

    showCharacterDetail(char) {
        this.selectedChar = char;
        
        const detailView = document.getElementById('detailView');
        const detailTitle = document.getElementById('detailTitle');
        const detailCanvas = document.getElementById('detailCanvas');
        const charDetails = document.getElementById('charDetails');

        detailTitle.textContent = `字符详情: ${char.char} (U+${char.unicode.toString(16).toUpperCase().padStart(4, '0')})`;
        
        this.drawCharacter(detailCanvas, char, this.currentZoom);

        charDetails.innerHTML = `
            <p><strong>Unicode:</strong> U+${char.unicode.toString(16).toUpperCase().padStart(4, '0')} (${char.unicode})</p>
            <p><strong>字符:</strong> ${char.char}</p>
            <p><strong>尺寸:</strong> ${char.width} × ${char.height} 像素</p>
            <p><strong>位图偏移:</strong> 0x${char.bitmapOffset.toString(16).toUpperCase()}</p>
            <p><strong>索引:</strong> ${char.index}</p>
        `;

        detailView.style.display = 'block';
        detailView.scrollIntoView({ behavior: 'smooth' });
    }

    exportCharacter() {
        if (!this.selectedChar) return;

        const canvas = document.createElement('canvas');
        const scale = 16; // 导出时使用更大的缩放
        this.drawCharacter(canvas, this.selectedChar, scale);

        const link = document.createElement('a');
        link.download = `char_${this.selectedChar.unicode}_${this.selectedChar.char}.png`;
        link.href = canvas.toDataURL();
        link.click();
    }

    showMessage(message, type) {
        const errorMsg = document.getElementById('errorMsg');
        const successMsg = document.getElementById('successMsg');
        const loadingMsg = document.getElementById('loadingMsg');

        // 隐藏所有消息
        errorMsg.style.display = 'none';
        successMsg.style.display = 'none';
        loadingMsg.style.display = 'none';

        if (type === 'error') {
            errorMsg.textContent = message;
            errorMsg.style.display = 'block';
        } else if (type === 'success') {
            successMsg.textContent = message;
            successMsg.style.display = 'block';
            setTimeout(() => successMsg.style.display = 'none', 3000);
        } else if (type === 'loading') {
            loadingMsg.innerHTML = `<h3>${message}</h3>`;
            loadingMsg.style.display = 'block';
        }
    }

    formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
}

// 初始化应用
document.addEventListener('DOMContentLoaded', () => {
    new FontBitmapAnalyzer();
});
