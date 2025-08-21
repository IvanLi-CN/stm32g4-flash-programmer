class FlashResourceViewer {
    constructor() {
        this.fontData = null;
        this.font12pxData = null;
        this.font16pxData = null;
        this.imageData = null;
        this.firmwareData = null;
        this.characters = [];
        this.currentCategory = 'all';
        this.currentZoom = 4;
        this.currentResourceType = 'firmware';
        this.currentFontType = '12px';
        this.comparisonMode = false;
        this.selectedChar = null;
        this.searchMode = 'char'; // 'char' 或 'unicode'
        
        this.initializeEventListeners();

        // 延迟初始化固件视图，确保DOM完全加载
        setTimeout(() => {
            this.initializeFirmwareView();
            // 自动加载默认固件
            this.loadDefaultFirmware();
        }, 500);
    }

    initializeEventListeners() {
        // 资源类型选择器
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
                e.target.classList.add('active');
                this.currentResourceType = e.target.dataset.type;
                this.switchResourceType();
            });
        });

        // 文件上传
        const uploadArea = document.getElementById('uploadArea');
        const uploadBtn = document.getElementById('uploadBtn');
        const fileInput = document.getElementById('fileInput');

        // 上传按钮点击事件
        uploadBtn.addEventListener('click', () => {
            fileInput.click();
        });

        // 上传区域点击事件
        uploadArea.addEventListener('click', (e) => {
            // 确保点击的不是按钮或文件输入元素
            if (e.target !== uploadBtn && e.target !== fileInput) {
                fileInput.click();
            }
        });
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

        // 字体选择器
        document.querySelectorAll('input[name="fontType"]').forEach(radio => {
            radio.addEventListener('change', (e) => {
                this.currentFontType = e.target.value;
                this.switchFont();
            });
        });

        // 字体对比功能
        const comparisonToggle = document.getElementById('comparisonToggle');
        comparisonToggle.addEventListener('click', () => {
            this.comparisonMode = !this.comparisonMode;
            comparisonToggle.textContent = this.comparisonMode ? '关闭字体对比' : '启用字体对比';
            comparisonToggle.classList.toggle('active', this.comparisonMode);
            this.updateFontDisplay();
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

        // 搜索选项卡
        document.querySelectorAll('.search-tab').forEach(tab => {
            tab.addEventListener('click', (e) => {
                document.querySelectorAll('.search-tab').forEach(t => t.classList.remove('active'));
                e.target.classList.add('active');
                this.searchMode = e.target.dataset.mode;
                this.updateSearchPlaceholder();
                // 重新执行搜索
                const searchBox = document.getElementById('searchBox');
                this.filterAndDisplayCharacters(searchBox.value);
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

        // 导出按钮 - 现在在详情面板中动态创建，不需要在这里绑定
    }

    switchResourceType() {
        // 隐藏所有预览容器
        document.querySelectorAll('.preview-containers').forEach(container => {
            container.classList.remove('active');
        });

        // 显示对应的预览容器
        const targetContainer = document.querySelector(`[data-viewer="${this.currentResourceType}"]`);
        if (targetContainer) {
            targetContainer.classList.add('active');
        }

        // 显示/隐藏相关控件
        const fontSelector = document.getElementById('fontSelector');
        const fontComparison = document.getElementById('fontComparison');
        const categories = document.getElementById('categories');
        const searchContainer = document.getElementById('searchContainer');
        const fontInfo = document.getElementById('fontInfo');

        if (this.currentResourceType === 'font') {
            fontSelector.style.display = 'block';
            fontComparison.style.display = 'block';
            categories.style.display = 'block';
            searchContainer.style.display = 'block';
            // fontInfo的显示由parseFontData控制
        } else if (this.currentResourceType === 'image') {
            fontSelector.style.display = 'none';
            fontComparison.style.display = 'none';
            categories.style.display = 'none';
            searchContainer.style.display = 'none';
            if (fontInfo) fontInfo.style.display = 'none';

            // 如果图片数据已加载，则渲染图片
            if (this.imageData) {
                this.parseRGB565Image();
            }
        } else {
            fontSelector.style.display = 'none';
            fontComparison.style.display = 'none';
            categories.style.display = 'none';
            searchContainer.style.display = 'none';
            if (fontInfo) fontInfo.style.display = 'none';
        }

        // 更新加载提示
        this.updateLoadingMessage();
    }

    initializeFirmwareView() {
        // 强制设置固件选项卡为激活状态
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.remove('active');
            if (btn.dataset.type === 'firmware') {
                btn.classList.add('active');
            }
        });

        // 显示固件结构（不需要实际的固件文件）
        this.parseFirmware();
        this.switchResourceType();
    }

    async loadDefaultFirmware() {
        try {
            // 优先尝试加载最新的固件文件
            let response = await fetch('./w25q128jv_complete.bin');
            if (!response.ok) {
                // 如果最新固件不存在，尝试加载旧版本
                response = await fetch('./pd-sink-128mbit.bin');
            }

            if (response.ok) {
                const arrayBuffer = await response.arrayBuffer();
                this.firmwareData = new Uint8Array(arrayBuffer);
                console.log('默认固件加载成功，大小:', this.firmwareData.length, '字节');

                // 自动解析所有资源
                this.autoLoadDefaultResources();
            } else {
                console.log('默认固件文件未找到，需要用户手动上传');
            }
        } catch (error) {
            console.log('加载默认固件失败:', error.message);
        }
    }

    autoLoadDefaultResources() {
        if (!this.firmwareData) return;

        try {
            // 自动提取启动画面图片数据
            const imageBlock = {
                name: 'boot_screen',
                address: 0x00000000,
                size: 110080,
                type: 'image',
                description: '启动画面 (320×172 RGB565)'
            };

            // 自动提取12px字体数据
            const font12pxBlock = {
                name: 'font_bitmap_12px',
                address: 0x00020000,
                size: 1048576,
                type: 'font',
                description: '12px字体位图'
            };

            // 自动提取16px字体数据
            const font16pxBlock = {
                name: 'font_bitmap_16px',
                address: 0x00120000,
                size: 1048576,
                type: 'font',
                description: '16px字体位图'
            };

            // 新增：Arial字体
            const arialFontBlock = {
                name: 'arial_font_16x24',
                address: 0x7D0000,
                size: 3716,
                type: 'font',
                description: 'Arial 16×24字体 (32-95)'
            };

            // 新增：Grotesk字体
            const groteskFontBlock = {
                name: 'grotesk_font_24x48',
                address: 0x7D2000,
                size: 9860,
                type: 'font',
                description: 'Grotesk Bold 24×48字体 (32-95)'
            };

            // 加载图片数据
            const imageData = this.firmwareData.slice(imageBlock.address, imageBlock.address + imageBlock.size);
            this.imageData = imageData;

            // 加载12px字体数据
            const font12pxData = this.firmwareData.slice(font12pxBlock.address, font12pxBlock.address + font12pxBlock.size);
            this.font12pxData = font12pxData;

            // 加载16px字体数据
            const font16pxData = this.firmwareData.slice(font16pxBlock.address, font16pxBlock.address + font16pxBlock.size);
            this.font16pxData = font16pxData;

            // 加载Arial字体数据
            const arialFontData = this.firmwareData.slice(arialFontBlock.address, arialFontBlock.address + arialFontBlock.size);
            this.arialFontData = arialFontData;

            // 加载Grotesk字体数据
            const groteskFontData = this.firmwareData.slice(groteskFontBlock.address, groteskFontBlock.address + groteskFontBlock.size);
            this.groteskFontData = groteskFontData;

            // 默认显示12px字体
            this.fontData = font12pxData;
            this.currentFontType = '12px';

            this.parseFont();
            this.updateFontUI();
            this.filterAndDisplayCharacters();

            this.showMessage(`已自动加载资源: 启动画面 + 12px(${this.characters.length}字符) + 16px字体 + Arial字体 + Grotesk字体！`, 'success');
        } catch (error) {
            console.error('自动加载资源失败:', error);
            this.showMessage('已加载默认固件，可以点击内存块预览内容', 'success');
        }
    }

    updateSearchPlaceholder() {
        const searchBox = document.getElementById('searchBox');
        if (this.searchMode === 'char') {
            searchBox.placeholder = '搜索字符...';
        } else {
            searchBox.placeholder = '搜索Unicode码点 (如: 4E2D 或 U+4E2D)...';
        }
    }

    updateLoadingMessage() {
        const loadingMsg = document.getElementById('loadingMsg');
        const uploadHint = document.getElementById('uploadHint');

        const messages = {
            'font': '请上传字体位图文件 (.bin)',
            'image': '请上传RGB565图片文件 (.bin)',
            'firmware': '请上传完整Flash固件文件 (.bin)'
        };

        const hints = {
            'font': '字体位图文件 (.bin)',
            'image': 'RGB565图片文件 (.bin)',
            'firmware': 'Flash固件文件 (.bin)'
        };

        loadingMsg.innerHTML = `
            <h3>👋 欢迎使用STM32G4 Flash资源预览器</h3>
            <p>${messages[this.currentResourceType]}</p>
        `;

        if (uploadHint) {
            uploadHint.textContent = hints[this.currentResourceType];
        }
    }

    detectFileType(file, data) {
        const ext = file.name.split('.').pop().toLowerCase();
        
        if (ext !== 'bin') {
            return 'unknown';
        }

        // 根据文件大小判断类型
        if (file.size === 16777216) { // 16MB
            return 'flash_firmware';
        } else if (file.size === 110080) { // 320x172x2 bytes
            return 'rgb565_image';
        } else if (file.size > 500000 && file.size < 800000) { // 字体文件大小范围
            // 检查是否有字符数量头部
            const charCount = data[0] | (data[1] << 8) | (data[2] << 16) | (data[3] << 24);
            if (charCount > 1000 && charCount < 50000) {
                return 'font_bitmap';
            }
        }

        return 'unknown';
    }

    async loadFile(file) {
        try {
            this.showMessage('正在加载文件...', 'loading');
            
            const arrayBuffer = await file.arrayBuffer();
            const data = new Uint8Array(arrayBuffer);
            
            const fileType = this.detectFileType(file, data);
            
            switch (fileType) {
                case 'font_bitmap':
                    await this.loadFontFile(file, data);
                    break;
                case 'rgb565_image':
                    await this.loadImageFile(file, data);
                    break;
                case 'flash_firmware':
                    await this.loadFirmwareFile(file, data);
                    break;
                default:
                    throw new Error('不支持的文件类型');
            }
            
        } catch (error) {
            this.showMessage(`加载文件失败: ${error.message}`, 'error');
        }
    }

    async loadFontFile(file, data) {
        // 根据当前选择的字体类型存储数据
        if (this.currentFontType === '12px') {
            this.font12pxData = data;
        } else {
            this.font16pxData = data;
        }

        this.fontData = data;
        this.parseFont();
        this.updateFontUI();
        this.filterAndDisplayCharacters();
        
        this.showMessage(`成功加载 ${this.currentFontType} 字体: ${this.characters.length} 个字符！`, 'success');
    }

    async loadImageFile(file, data) {
        this.imageData = data;
        this.parseRGB565Image();
        this.showMessage(`成功加载RGB565图片: ${file.name}`, 'success');
    }

    async loadFirmwareFile(file, data) {
        this.firmwareData = data;
        this.parseFirmware();
        this.showMessage(`成功加载Flash固件: ${file.name}`, 'success');
    }

    parseRGB565Image() {
        const canvas = document.getElementById('imageCanvas');
        const ctx = canvas.getContext('2d');
        
        // RGB565图片尺寸
        const width = 320;
        const height = 172;
        
        canvas.width = width;
        canvas.height = height;
        
        const imageData = ctx.createImageData(width, height);
        const data = imageData.data;
        
        // 解析RGB565数据
        for (let i = 0; i < this.imageData.length; i += 2) {
            const rgb565 = this.imageData[i] | (this.imageData[i + 1] << 8);
            
            // 提取RGB分量
            const r = ((rgb565 >> 11) & 0x1F) << 3; // 5位红色
            const g = ((rgb565 >> 5) & 0x3F) << 2;  // 6位绿色
            const b = (rgb565 & 0x1F) << 3;         // 5位蓝色
            
            const pixelIndex = (i / 2) * 4;
            data[pixelIndex] = r;     // R
            data[pixelIndex + 1] = g; // G
            data[pixelIndex + 2] = b; // B
            data[pixelIndex + 3] = 255; // A
        }
        
        ctx.putImageData(imageData, 0, 0);
        
        // 更新图片信息
        const imageInfo = document.getElementById('imageInfo');
        document.getElementById('imageDimensions').textContent = `${width} × ${height}`;
        document.getElementById('imageFileSize').textContent = `${(this.imageData.length / 1024).toFixed(1)} KB`;
        imageInfo.style.display = 'block';

        // 在详情面板显示图片信息
        this.showImageDetails(width, height, this.imageData.length);
    }

    parseFirmware() {
        // 根据内存布局解析固件
        const memoryLayout = [
            { name: 'boot_screen', address: 0x00000000, size: 110080, type: 'image', description: '启动画面 (320×172 RGB565)' },
            { name: 'font_bitmap_12px', address: 0x00020000, size: 1048576, type: 'font', description: '12px字体位图' },
            { name: 'font_bitmap_16px', address: 0x00120000, size: 1048576, type: 'font', description: '16px字体位图' },
            { name: 'arial_font_16x24', address: 0x7D0000, size: 3716, type: 'font', description: 'Arial 16×24字体 (32-95)' },
            { name: 'grotesk_font_24x48', address: 0x7D2000, size: 9860, type: 'font', description: 'Grotesk Bold 24×48字体 (32-95)' }
        ];

        const memoryBlocks = document.getElementById('memoryBlocks');
        memoryBlocks.innerHTML = '';

        memoryLayout.forEach(block => {
            const blockElement = document.createElement('div');
            blockElement.className = `memory-block ${block.name.replace('_', '-')}`;
            blockElement.innerHTML = `
                <div class="block-info">
                    <div class="block-name">${block.description}</div>
                    <div class="block-details">
                        地址: 0x${block.address.toString(16).toUpperCase().padStart(8, '0')} - 
                        0x${(block.address + block.size - 1).toString(16).toUpperCase().padStart(8, '0')}
                        (${(block.size / 1024).toFixed(0)} KB)
                    </div>
                </div>
            `;
            
            blockElement.addEventListener('click', () => {
                this.showMemoryBlockDetails(block);
                this.extractAndPreviewResource(block);
            });
            
            memoryBlocks.appendChild(blockElement);
        });
    }

    extractAndPreviewResource(block) {
        // 检查是否有固件数据
        if (!this.firmwareData) {
            this.showMessage('请先上传完整的Flash固件文件 (.bin) 才能预览资源内容', 'error');
            return;
        }

        const resourceData = this.firmwareData.slice(block.address, block.address + block.size);
        
        if (block.type === 'image') {
            // 切换到图片预览模式
            this.currentResourceType = 'image';
            document.querySelector('[data-type="image"]').click();
            this.imageData = resourceData;
            this.parseRGB565Image();
        } else if (block.type === 'font') {
            // 切换到字体预览模式
            this.currentResourceType = 'font';
            document.querySelector('[data-type="font"]').click();

            if (block.name.includes('12px')) {
                this.currentFontType = '12px';
                document.getElementById('font12px').checked = true;
                this.font12pxData = resourceData;
            } else if (block.name.includes('16px')) {
                this.currentFontType = '16px';
                document.getElementById('font16px').checked = true;
                this.font16pxData = resourceData;
            } else if (block.name.includes('arial_font')) {
                this.currentFontType = 'arial';
                document.getElementById('fontArial').checked = true;
                this.arialFontData = resourceData;
            } else if (block.name.includes('grotesk_font')) {
                this.currentFontType = 'grotesk';
                document.getElementById('fontGrotesk').checked = true;
                this.groteskFontData = resourceData;
            }

            this.fontData = resourceData;
            this.parseFont();
            this.updateFontUI();
            this.filterAndDisplayCharacters();
        }
        
        this.showMessage(`已提取并预览: ${block.description}`, 'success');
    }

    showMemoryBlockDetails(block) {
        const detailsPanel = document.getElementById('detailsPanel');

        detailsPanel.innerHTML = `
            <h3>💾 内存块详情</h3>
            <div class="memory-block-info">
                <div class="info-item"><strong>名称:</strong> ${block.description}</div>
                <div class="info-item"><strong>地址:</strong> 0x${block.address.toString(16).toUpperCase().padStart(8, '0')}</div>
                <div class="info-item"><strong>大小:</strong> ${(block.size / 1024).toFixed(2)} KB</div>
                <div class="info-item"><strong>类型:</strong> ${block.type === 'image' ? '图片' : '字体'}</div>
                <div class="info-item"><strong>范围:</strong> 0x${block.address.toString(16).toUpperCase().padStart(8, '0')} - 0x${(block.address + block.size - 1).toString(16).toUpperCase().padStart(8, '0')}</div>
            </div>
            <div style="margin-top: 15px; padding: 10px; background: #e7f3ff; border-radius: 4px; font-size: 12px;">
                <strong>💡 提示:</strong> 点击此内存块将自动切换到对应的预览模式并加载数据
            </div>
        `;
    }

    showImageDetails(width, height, dataSize) {
        const detailsPanel = document.getElementById('detailsPanel');

        detailsPanel.innerHTML = `
            <h3>🖼️ 图片详情</h3>
            <div class="image-info">
                <div class="info-item"><strong>尺寸:</strong> ${width} × ${height} 像素</div>
                <div class="info-item"><strong>格式:</strong> RGB565</div>
                <div class="info-item"><strong>文件大小:</strong> ${(dataSize / 1024).toFixed(1)} KB</div>
                <div class="info-item"><strong>像素总数:</strong> ${(width * height).toLocaleString()}</div>
                <div class="info-item"><strong>色彩深度:</strong> 16位 (65,536色)</div>
                <div class="info-item"><strong>数据大小:</strong> ${dataSize.toLocaleString()} 字节</div>
            </div>
            <div style="margin-top: 15px; padding: 10px; background: #e7f3ff; border-radius: 4px; font-size: 12px;">
                <strong>💡 RGB565格式说明:</strong><br>
                • 红色: 5位 (32级)<br>
                • 绿色: 6位 (64级)<br>
                • 蓝色: 5位 (32级)<br>
                • 每像素占用2字节
            </div>
        `;
    }

    parseFont() {
        if (!this.fontData || this.fontData.length < 4) {
            throw new Error('文件太小，不是有效的字体文件');
        }

        // 读取字符数量 (小端序)
        const charCount = this.fontData[0] |
                         (this.fontData[1] << 8) |
                         (this.fontData[2] << 16) |
                         (this.fontData[3] << 24);

        if (charCount <= 0 || charCount > 50000) {
            throw new Error(`字符数量异常: ${charCount}`);
        }

        this.characters = [];
        let offset = 4; // 跳过头部

        // 读取字符信息表 (10字节结构)
        for (let i = 0; i < charCount; i++) {
            if (offset + 10 > this.fontData.length) {
                break;
            }

            const unicode = this.fontData[offset] |
                           (this.fontData[offset + 1] << 8) |
                           (this.fontData[offset + 2] << 16) |
                           (this.fontData[offset + 3] << 24);

            const width = this.fontData[offset + 4];
            const height = this.fontData[offset + 5];

            const bitmapOffset = this.fontData[offset + 6] |
                               (this.fontData[offset + 7] << 8) |
                               (this.fontData[offset + 8] << 16) |
                               (this.fontData[offset + 9] << 24);

            this.characters.push({
                unicode: unicode,
                char: String.fromCharCode(unicode),
                width: width,
                height: height,
                bitmapOffset: bitmapOffset,
                index: i
            });

            offset += 10;
        }

        console.log(`解析完成: ${this.characters.length} 个字符`);
    }

    updateFontUI() {
        const fontInfo = document.getElementById('fontInfo');
        const fontSelector = document.getElementById('fontSelector');
        const fontComparison = document.getElementById('fontComparison');
        const categories = document.getElementById('categories');
        const searchBox = document.getElementById('searchBox');

        // 显示字体信息
        document.getElementById('charCount').textContent = this.characters.length.toLocaleString();
        document.getElementById('fileSize').textContent = `${(this.fontData.length / 1024).toFixed(2)} KB`;

        // 只在字体模式下显示字体相关控件
        if (this.currentResourceType === 'font') {
            fontInfo.style.display = 'block';
            fontSelector.style.display = 'block';
            fontComparison.style.display = 'block';
            categories.style.display = 'block';
            searchBox.style.display = 'block';
        }
    }

    switchFont() {
        let targetData;
        let fontName;

        switch (this.currentFontType) {
            case '12px':
                targetData = this.font12pxData;
                fontName = '12px 字体';
                break;
            case '16px':
                targetData = this.font16pxData;
                fontName = '16px 字体';
                break;
            case 'arial':
                targetData = this.arialFontData;
                fontName = 'Arial 16×24字体';
                break;
            case 'grotesk':
                targetData = this.groteskFontData;
                fontName = 'Grotesk Bold 24×48字体';
                break;
            default:
                targetData = this.font12pxData;
                fontName = '12px 字体';
        }

        if (!targetData) {
            this.showMessage(`请先加载 ${fontName} 文件`, 'error');
            return;
        }

        this.fontData = targetData;
        this.parseFont();
        this.updateFontUI();
        this.filterAndDisplayCharacters();

        this.showMessage(`已切换到 ${fontName}`, 'success');
    }

    updateFontDisplay() {
        if (this.comparisonMode && this.font12pxData && this.font16pxData) {
            // 对比模式：显示两种字体
            this.displayComparisonView();
        } else {
            // 普通模式：显示当前字体
            this.filterAndDisplayCharacters();
        }
    }

    displayComparisonView() {
        // 实现字体对比视图
        const charGrid = document.getElementById('charGrid');
        charGrid.innerHTML = '<h4>字体对比模式 - 功能开发中...</h4>';
        charGrid.style.display = 'block';
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
            if (this.searchMode === 'char') {
                // 字符搜索模式
                const term = searchTerm.toLowerCase();
                filteredChars = filteredChars.filter(char => {
                    return char.char.toLowerCase().includes(term);
                });
            } else {
                // Unicode码点搜索模式
                const term = searchTerm.replace(/^U\+/i, '').toLowerCase();
                filteredChars = filteredChars.filter(char => {
                    const hexCode = char.unicode.toString(16).toLowerCase();
                    const decCode = char.unicode.toString();
                    return hexCode.includes(term) || decCode.includes(term);
                });
            }
        }

        this.displayCharacters(filteredChars);
    }

    displayCharacters(characters) {
        const charGrid = document.getElementById('charGrid');

        if (characters.length === 0) {
            charGrid.innerHTML = '<p>没有找到匹配的字符</p>';
            charGrid.style.display = 'block';
            return;
        }

        // 使用虚拟列表显示所有字符
        this.initVirtualGrid(charGrid, characters);
        charGrid.style.display = 'block';
    }

    initVirtualGrid(container, characters) {
        // 清空容器
        container.innerHTML = '';

        // 创建虚拟网格容器
        const virtualContainer = document.createElement('div');
        virtualContainer.className = 'virtual-grid-container';
        virtualContainer.style.position = 'relative';
        virtualContainer.style.height = '100%';
        virtualContainer.style.overflow = 'auto';

        // 网格配置
        const itemWidth = 116; // 100px + 12px gap + 4px padding
        const itemHeight = 140; // 字符项目高度
        const containerWidth = container.offsetWidth || 800;
        const columnsPerRow = Math.floor((containerWidth - 20) / itemWidth) || 8; // 减去padding
        const totalRows = Math.ceil(characters.length / columnsPerRow);
        const totalHeight = totalRows * itemHeight + 20; // 加上padding

        // 创建可见区域（占位符）
        const viewport = document.createElement('div');
        viewport.className = 'virtual-grid-viewport';
        viewport.style.position = 'relative';
        viewport.style.height = `${totalHeight}px`;
        viewport.style.width = '100%';

        // 创建可见项目容器
        const visibleItems = document.createElement('div');
        visibleItems.className = 'virtual-grid-items';
        visibleItems.style.position = 'absolute';
        visibleItems.style.top = '0';
        visibleItems.style.left = '0';
        visibleItems.style.width = '100%';
        visibleItems.style.display = 'grid';
        visibleItems.style.gridTemplateColumns = `repeat(${columnsPerRow}, 1fr)`;
        visibleItems.style.gap = '12px';
        visibleItems.style.padding = '10px';

        viewport.appendChild(visibleItems);
        virtualContainer.appendChild(viewport);
        container.appendChild(virtualContainer);

        // 虚拟滚动逻辑
        let lastScrollTop = 0;
        const bufferRows = 2; // 缓冲行数

        const updateVisibleItems = () => {
            const scrollTop = virtualContainer.scrollTop;
            const containerHeight = virtualContainer.clientHeight;

            // 计算可见区域
            const startRow = Math.max(0, Math.floor(scrollTop / itemHeight) - bufferRows);
            const endRow = Math.min(totalRows - 1, Math.ceil((scrollTop + containerHeight) / itemHeight) + bufferRows);

            const startIndex = startRow * columnsPerRow;
            const endIndex = Math.min(characters.length - 1, (endRow + 1) * columnsPerRow - 1);

            // 清空并重新渲染可见项目
            visibleItems.innerHTML = '';
            visibleItems.style.top = `${startRow * itemHeight}px`;

            for (let i = startIndex; i <= endIndex; i++) {
                if (i >= characters.length) break;

                const char = characters[i];
                const item = this.createCharacterItem(char);
                visibleItems.appendChild(item);
            }

            console.log(`虚拟滚动: 显示 ${startIndex}-${endIndex} (${endIndex - startIndex + 1}个字符)`);
            lastScrollTop = scrollTop;
        };

        // 初始渲染
        updateVisibleItems();

        // 添加滚动事件监听器
        let scrollTimeout;
        virtualContainer.addEventListener('scroll', () => {
            // 使用防抖避免频繁更新
            clearTimeout(scrollTimeout);
            scrollTimeout = setTimeout(() => {
                updateVisibleItems();
            }, 16); // 约60fps
        });

        // 添加窗口大小变化监听器
        const resizeObserver = new ResizeObserver(() => {
            // 重新计算布局
            const newContainerWidth = container.offsetWidth || 800;
            const newColumnsPerRow = Math.floor((newContainerWidth - 20) / itemWidth) || 8;

            if (newColumnsPerRow !== columnsPerRow) {
                // 如果列数变化，重新初始化虚拟滚动
                this.renderCharacterGrid(characters, container);
            }
        });

        resizeObserver.observe(container);
    }

    createCharacterItem(char) {
        const item = document.createElement('div');
        item.className = 'char-item';
        item.addEventListener('click', () => this.showCharacterDetail(char));

        // 创建固定尺寸的canvas容器
        const canvasContainer = document.createElement('div');
        canvasContainer.className = 'char-canvas-container';

        // 创建canvas，尺寸完全匹配字符位图
        const canvas = document.createElement('canvas');
        canvas.className = 'char-canvas';
        const scale = 1;  // 使用1倍缩放显示实际字形大小
        canvas.width = char.width * scale;
        canvas.height = char.height * scale;

        this.drawCharacter(canvas, char, scale);

        const unicode = document.createElement('div');
        unicode.className = 'char-unicode';
        unicode.textContent = `U+${char.unicode.toString(16).toUpperCase().padStart(4, '0')}`;

        const size = document.createElement('div');
        size.className = 'char-size';
        size.textContent = `${char.width}×${char.height}px`;

        canvasContainer.appendChild(canvas);
        item.appendChild(canvasContainer);
        item.appendChild(unicode);
        item.appendChild(size);

        return item;
    }

    drawCharacter(canvas, char, scale = 1) {
        const ctx = canvas.getContext('2d');
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        if (!this.fontData || char.bitmapOffset >= this.fontData.length) {
            return;
        }

        const bytesPerRow = Math.ceil(char.width / 8);
        const bitmapSize = bytesPerRow * char.height;

        if (char.bitmapOffset + bitmapSize > this.fontData.length) {
            return;
        }

        // 绘制字符位图（canvas尺寸已完全匹配字符尺寸）
        for (let y = 0; y < char.height; y++) {
            for (let x = 0; x < char.width; x++) {
                const byteIndex = char.bitmapOffset + y * bytesPerRow + Math.floor(x / 8);
                const bitIndex = 7 - (x % 8);
                const pixel = (this.fontData[byteIndex] >> bitIndex) & 1;

                if (pixel) {
                    ctx.fillStyle = '#000';
                    ctx.fillRect(
                        x * scale,
                        y * scale,
                        scale,
                        scale
                    );
                }
            }
        }
    }

    showCharacterDetail(char) {
        this.selectedChar = char;

        const detailsPanel = document.getElementById('detailsPanel');

        // 设置画布大小
        const scale = this.currentZoom;

        // 在详情面板显示字符信息
        detailsPanel.innerHTML = `
            <h3>📋 字符详情</h3>
            <div class="character-details-layout">
                <div class="character-preview-column">
                    <div class="character-preview">
                        <canvas id="detailCanvas" width="${char.width * scale}" height="${char.height * scale}" style="border: 1px solid #ddd; background: white;"></canvas>
                    </div>
                    <div class="zoom-controls">
                        <button class="zoom-btn ${scale === 1 ? 'active' : ''}" data-zoom="1">1x</button>
                        <button class="zoom-btn ${scale === 4 ? 'active' : ''}" data-zoom="4">4x</button>
                        <button class="zoom-btn ${scale === 8 ? 'active' : ''}" data-zoom="8">8x</button>
                        <button class="zoom-btn ${scale === 16 ? 'active' : ''}" data-zoom="16">16x</button>
                    </div>
                </div>
                <div class="character-info-column">
                    <div class="character-info">
                        <div class="info-item"><strong>字符:</strong> <span>"${char.char}"</span></div>
                        <div class="info-item"><strong>Unicode:</strong> <span>U+${char.unicode.toString(16).toUpperCase().padStart(4, '0')} (${char.unicode})</span></div>
                        <div class="info-item"><strong>尺寸:</strong> <span>${char.width} × ${char.height} 像素</span></div>
                        <div class="info-item"><strong>位图偏移:</strong> <span>0x${char.bitmapOffset.toString(16).toUpperCase()}</span></div>
                        <div class="info-item"><strong>索引:</strong> <span>${char.index}</span></div>
                    </div>
                    <button class="export-btn" id="exportCharBtn">导出为PNG</button>
                </div>
            </div>
        `;

        // 重新绑定缩放按钮事件
        const zoomBtns = detailsPanel.querySelectorAll('.zoom-btn');
        zoomBtns.forEach(btn => {
            btn.addEventListener('click', (e) => {
                this.currentZoom = parseInt(e.target.dataset.zoom);
                this.showCharacterDetail(char); // 重新渲染
            });
        });

        // 绑定导出按钮事件
        const exportBtn = document.getElementById('exportCharBtn');
        if (exportBtn) {
            exportBtn.addEventListener('click', () => {
                this.exportCharacter(char);
            });
        }

        // 绘制字符到canvas
        const detailCanvas = document.getElementById('detailCanvas');
        this.drawCharacter(detailCanvas, char, scale);
    }

    exportCharacter(char) {
        const canvas = document.createElement('canvas');
        canvas.width = char.width * 8; // 8倍放大
        canvas.height = char.height * 8;

        this.drawCharacter(canvas, char, 8);

        // 下载图片
        const link = document.createElement('a');
        link.download = `char_${char.unicode}_${char.char}.png`;
        link.href = canvas.toDataURL();
        link.click();
    }

    showMessage(message, type) {
        const loadingMsg = document.getElementById('loadingMsg');
        const errorMsg = document.getElementById('errorMsg');
        const successMsg = document.getElementById('successMsg');

        // 隐藏所有消息
        loadingMsg.style.display = 'none';
        errorMsg.style.display = 'none';
        successMsg.style.display = 'none';

        switch (type) {
            case 'loading':
                loadingMsg.innerHTML = `<h3>${message}</h3>`;
                loadingMsg.style.display = 'block';
                break;
            case 'error':
                errorMsg.textContent = message;
                errorMsg.style.display = 'block';
                break;
            case 'success':
                successMsg.textContent = message;
                successMsg.style.display = 'block';
                setTimeout(() => {
                    successMsg.style.display = 'none';
                }, 3000);
                break;
        }
    }
}

// 初始化应用
document.addEventListener('DOMContentLoaded', () => {
    window.flashViewer = new FlashResourceViewer();
});
