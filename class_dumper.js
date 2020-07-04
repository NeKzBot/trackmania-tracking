const fs = require('fs');

const game = process.argv[2];
const exe = process.argv[3];

const offsets = (m_iSize, m_szClassName, m_iClassId) => ({ m_iSize, m_szClassName, m_iClassId  });
const pattern = (signature, offsets) => ({ signature, offsets });

const patterns = [
    pattern(
        [
            0x48, 0x83, 0xEC, 0x38,
            0x48, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
            0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, // mov [rsp+38h+var_10], m_iSize
            0x4C, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,       // lea r9, m_szClassName
            0x48, 0x89, 0x44, 0x24, 0x00,
            0x4C, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
            0xBA, 0x00, 0x00, 0x00, 0x00,                   // mov edx, m_iClassId
            /* 0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
            0xE8, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x83, 0xC4, 0x38,
            0xE9, */
        ],
        offsets(15, 22, 39),
    ),
    pattern(
        [
            0x48, 0x83, 0xEC, 0x38,
            0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,       // mov [rsp+38h+var_10], m_iSize
            0x4C, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,             // lea r9, m_szClassName
            0x4C, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
            0x48, 0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xBA, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,             //  mov edx, m_iClassId
            0xE8, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x89, 0x0D, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x83, 0xC4, 0x38,
            0xE9, 0x00, 0x00, 0x00, 0x00,
        ],
        offsets(8, 15, 43),
    ),
    pattern(
        [
            0x48, 0x83, 0xEC, 0x38,
            0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,       // mov [rsp+38h+var_10], m_iSize
            0x4C, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,             // lea r9, m_szClassName
            0x4C, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
            0x48, 0xC7, 0x44, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xBA, 0x00, 0x00, 0x00, 0x00,                         //  mov edx, m_iClassId
            0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
            0xE8, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x8D, 0x0D, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x83, 0xC4, 0x38,
            0xE9, 0x00, 0x00, 0x00, 0x00,
        ],
        offsets(8, 15, 36),
    ),
];

const buffer = fs.readFileSync(exe);
const bufferLength = buffer.length;

const i32 = (offset) => {
    return buffer[offset]
        + (buffer[offset + 1] << 8)
        + (buffer[offset + 2] << 16)
        + (buffer[offset + 3] << 24);
};

const cstr = (offset) => {
    let result = '';
    while (buffer[offset] !== 0x00) {
        result += String.fromCharCode(buffer[offset++]);
    }
    return result;
};

const deref = (type, offset) => {
    return type(offset + i32(offset) + 4);
};

const classes = [];

for (const { signature, offsets } of patterns) {
    const signatureLength = signature.length;

    for (let byteIndex = 0; byteIndex < bufferLength; ++byteIndex) {
        let matches = 0;
        let sigIndex = 0;

        for (const byte of signature) {
            if (byte !== 0x00 && byte !== buffer[byteIndex + sigIndex]) {
                break;
            }
            ++matches;
            ++sigIndex;
        }

        if (matches === signatureLength) {
            const m_iSize = i32(byteIndex + offsets.m_iSize);
            const m_szClassName = deref(cstr, byteIndex + offsets.m_szClassName);
            const m_iClassId = i32(byteIndex + offsets.m_iClassId);

            classes.push({ m_iSize, m_szClassName, m_iClassId });
        }
    }
}

//const by = (key) => (a, b) => b[key] - a[key];
const byString = (key) => (a, b) => a[key].localeCompare(b[key]);

console.log(`# ${game} Classes\r\n`);
console.log(`Dumped game classes from ${exe.substr(exe.lastIndexOf('/') + 1)}.\r\n`);
console.log('|Class|Id|Size|\r\n|---|:-:|:-:|');

for (const { m_szClassName, m_iClassId, m_iSize } of classes.sort(byString('m_szClassName'))) {
    console.log(`|${m_szClassName}|0x${m_iClassId.toString(16)}|${m_iSize}|`);
}
