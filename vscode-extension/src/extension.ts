import * as vscode from 'vscode';
import { spawn } from 'child_process';
import * as path from 'path';

let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('DevUtils');
    context.subscriptions.push(outputChannel);

    // Register commands
    let autonomous = vscode.commands.registerCommand('devutils.autonomous', async () => {
        const task = await vscode.window.showInputBox({
            prompt: 'What task should the AI agent perform?',
            placeHolder: 'e.g., add error handling to file reading'
        });

        if (task) {
            runAutonomousAgent(task);
        }
    });

    let chat = vscode.commands.registerCommand('devutils.chat', async () => {
        const message = await vscode.window.showInputBox({
            prompt: 'What would you like to ask the AI?',
            placeHolder: 'e.g., how do I read a file in Rust?'
        });

        if (message) {
            runChat(message);
        }
    });

    let generate = vscode.commands.registerCommand('devutils.generate', async () => {
        const prompt = await vscode.window.showInputBox({
            prompt: 'What code should I generate?',
            placeHolder: 'e.g., a function that calculates fibonacci'
        });

        if (prompt) {
            runGenerate(prompt);
        }
    });

    let explain = vscode.commands.registerCommand('devutils.explain', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }

        const selected = editor.document.getText(editor.selection);
        if (!selected) {
            vscode.window.showErrorMessage('No text selected');
            return;
        }

        runExplain(selected);
    });

    let fix = vscode.commands.registerCommand('devutils.fix', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }

        const selected = editor.document.getText(editor.selection);
        if (!selected) {
            vscode.window.showErrorMessage('No text selected');
            return;
        }

        runFix(selected);
    });

    context.subscriptions.push(autonomous, chat, generate, explain, fix);

    outputChannel.appendLine('DevUtils extension activated');
}

async function runAutonomousAgent(task: string) {
    outputChannel.show(true);
    outputChannel.appendLine(`🤖 Starting autonomous agent: ${task}`);

    try {
        const result = await runDevUtilsCli(['autonomous', task]);
        outputChannel.appendLine(result);
        vscode.window.showInformationMessage('Autonomous agent completed!');
    } catch (error: any) {
        outputChannel.appendLine(`Error: ${error.message}`);
        vscode.window.showErrorMessage('Autonomous agent failed');
    }
}

async function runChat(message: string) {
    outputChannel.show(true);
    outputChannel.appendLine(`💬 Chat: ${message}`);

    try {
        const result = await runDevUtilsCli(['ai', 'chat', message]);
        outputChannel.appendLine(result);
    } catch (error: any) {
        outputChannel.appendLine(`Error: ${error.message}`);
    }
}

async function runGenerate(prompt: string) {
    outputChannel.show(true);
    outputChannel.appendLine(`✨ Generating: ${prompt}`);

    try {
        const result = await runDevUtilsCli(['generate', prompt]);
        
        // Ask if user wants to insert the code
        const action = await vscode.window.showQuickPick(['Insert at cursor', 'Copy to clipboard', 'New file'], {
            placeHolder: 'What would you like to do with the generated code?'
        });

        if (action === 'Insert at cursor') {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                editor.edit(editBuilder => {
                    editBuilder.insert(editor.selection.active, result);
                });
            }
        } else if (action === 'Copy to clipboard') {
            vscode.env.clipboard.writeText(result);
            vscode.window.showInformationMessage('Copied to clipboard');
        } else if (action === 'New file') {
            const doc = await vscode.workspace.openTextDocument({
                content: result,
                language: 'typescript'
            });
            await vscode.window.showTextDocument(doc);
        }

        outputChannel.appendLine(result);
    } catch (error: any) {
        outputChannel.appendLine(`Error: ${error.message}`);
    }
}

async function runExplain(code: string) {
    outputChannel.show(true);
    outputChannel.appendLine('📖 Explaining code...');

    try {
        const result = await runDevUtilsCli(['explain', code]);
        outputChannel.appendLine(result);
        
        // Show in separate panel
        const panel = vscode.window.createWebviewPanel(
            'devutils.explain',
            'DevUtils - Code Explanation',
            vscode.ViewColumn.Beside,
            {}
        );

        panel.webview.html = `
            <!DOCTYPE html>
            <html>
            <body>
                <h2>Code Explanation</h2>
                <pre>${escapeHtml(result)}</pre>
            </body>
            </html>
        `;
    } catch (error: any) {
        outputChannel.appendLine(`Error: ${error.message}`);
    }
}

async function runFix(code: string) {
    outputChannel.show(true);
    outputChannel.appendLine('🔧 Fixing code...');

    try {
        const result = await runDevUtilsCli(['ai', 'fix', code]);
        outputChannel.appendLine(result);

        const editor = vscode.window.activeTextEditor;
        if (editor) {
            const action = await vscode.window.showQuickPick(['Replace selection', 'Show diff'], {
                placeHolder: 'How would you like to apply the fix?'
            });

            if (action === 'Replace selection') {
                editor.edit(editBuilder => {
                    editBuilder.replace(editor.selection, result);
                });
            }
        }
    } catch (error: any) {
        outputChannel.appendLine(`Error: ${error.message}`);
    }
}

function runDevUtilsCli(args: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
        const devutilsPath = vscode.workspace.getConfiguration('devutils').get('binaryPath', 'devutils');
        const child = spawn(devutilsPath, args, {
            env: {
                ...process.env,
                OPENAI_API_KEY: vscode.workspace.getConfiguration('devutils').get('apiKey') || process.env.OPENAI_API_KEY || '',
            }
        });

        let output = '';
        let errorOutput = '';

        child.stdout.on('data', (data) => {
            output += data.toString();
        });

        child.stderr.on('data', (data) => {
            errorOutput += data.toString();
        });

        child.on('close', (code) => {
            if (code === 0) {
                resolve(output);
            } else {
                reject(new Error(errorOutput || 'Command failed'));
            }
        });

        child.on('error', (error) => {
            reject(error);
        });
    });
}

function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}

export function deactivate() {}
