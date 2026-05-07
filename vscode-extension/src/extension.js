const { spawn } = require('child_process');

function activate(context) {
    const commands = [
        { cmd: 'chat', title: 'DevUtils: Chat' },
        { cmd: 'search', title: 'DevUtils: Search' },
        { cmd: 'generate', title: 'DevUtils: Generate' },
        { cmd: 'explain', title: 'DevUtils: Explain' },
        { cmd: 'refactor', title: 'DevUtils: Refactor' },
        { cmd: 'test', title: 'DevUtils: Generate Tests' },
        { cmd: 'hooks list', title: 'DevUtils: List Hooks' },
        { cmd: 'skills list', title: 'DevUtils: List Skills' },
        { cmd: 'mcp list', title: 'DevUtils: List MCP Servers' },
    ];

    const vscode = require('vscode');

    commands.forEach(({ cmd, title }) => {
        context.subscriptions.push(
            vscode.commands.registerCommand(`devutils.${cmd.replace(' ', '.')}`, async () => {
                const input = cmd.includes(' ') ? await vscode.window.showInputBox({
                    prompt: `Enter ${cmd}...`
                }) : null;

                const args = input ? ['ai', cmd.split(' ')[0], input] : [cmd.includes(' ') ? cmd.split(' ')[0] : cmd];
                const exe = findDevUtils();
                
                if (!exe) {
                    vscode.window.showErrorMessage('DevUtils not found');
                    return;
                }

                const output = await new Promise((resolve, reject) => {
                    const proc = spawn(exe, args);
                    let data = '';
                    proc.stdout.on('data', d => data += d);
                    proc.on('close', c => c === 0 ? resolve(data) : reject(c));
                    proc.on('error', reject);
                });

                vscode.window.showInformationMessage(output.substring(0, 100));
            })
        );
    });
}

function findDevUtils() {
    const paths = [
        'C:\\Users\\arvin\\Documents\\OPENCODE\\devutils_rs\\target\\release\\devutils.exe',
        'C:\\Users\\arvin\\Documents\\OPENCODE\\devutils_rs\\target\\debug\\devutils.exe',
    ];
    const fs = require('fs');
    for (const p of paths) if (fs.existsSync(p)) return p;
    return 'devutils';
}

function deactivate() {}

module.exports = { activate, deactivate };