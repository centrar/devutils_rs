"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const child_process_1 = require("child_process");
const path = __importStar(require("path"));
function activate(context) {
    context.subscriptions.push(vscode.commands.registerCommand('devutils.chat', () => chat()), vscode.commands.registerCommand('devutils.search', () => search()), vscode.commands.registerCommand('devutils.generate', () => generate()), vscode.commands.registerCommand('devutils.explain', () => explain()), vscode.commands.registerCommand('devutils.refactor', () => refactor()), vscode.commands.registerCommand('devutils.test', () => generateTests()), vscode.commands.registerCommand('devutils.hooks.list', () => listHooks()), vscode.commands.registerCommand('devutils.skills.list', () => listSkills()), vscode.commands.registerCommand('devutils.mcp.list', () => listMCP()), vscode.commands.registerCommand('devutils.mcp.install', () => installMCP()));
    registerCompletions(context);
    registerHover(context);
}
function deactivate() { }
async function chat() {
    const input = await vscode.window.showInputBox({
        prompt: 'Ask DevUtils...',
        placeHolder: 'Describe what you want to build...'
    });
    if (input) {
        await runDevUtilsCommand(['ai', 'chat', input]);
    }
}
async function search() {
    const input = await vscode.window.showInputBox({
        prompt: 'Search code semantically...',
        placeHolder: 'e.g., "user authentication logic"'
    });
    if (input) {
        await runDevUtilsCommand(['search', input]);
    }
}
async function generate() {
    const input = await vscode.window.showInputBox({
        prompt: 'Generate code...',
        placeHolder: 'Describe what to generate'
    });
    if (input) {
        await runDevUtilsCommand(['generate', input]);
    }
}
async function explain() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('No active editor');
        return;
    }
    const selection = editor.selection;
    const selectedText = editor.document.getText(selection);
    if (!selectedText) {
        vscode.window.showWarningMessage('Select code to explain');
        return;
    }
    await runDevUtilsCommand(['explain', selectedText]);
}
async function refactor() {
    const editor = vscode.window.activeTextEditor;
    if (!editor)
        return;
    const selectedText = editor.document.getText(editor.selection);
    if (selectedText) {
        await runDevUtilsCommand(['generate', `Refactor: ${selectedText}`]);
    }
}
async function generateTests() {
    const editor = vscode.window.activeTextEditor;
    if (!editor)
        return;
    const selectedText = editor.document.getText(editor.selection);
    if (selectedText) {
        await runDevUtilsCommand(['tests', selectedText]);
    }
}
async function listHooks() {
    await runDevUtilsCommand(['hooks', 'list']);
}
async function listSkills() {
    await runDevUtilsCommand(['skills', 'list']);
}
async function listMCP() {
    await runDevUtilsCommand(['mcp', 'list']);
}
async function installMCP() {
    const input = await vscode.window.showInputBox({
        prompt: 'Install MCP server...',
        placeHolder: 'Server name'
    });
    if (input) {
        await runDevUtilsCommand(['mcp', 'install', input]);
    }
}
function registerCompletions(context) {
    const provider = {
        provideCompletionItems: async (document, position) => {
            const line = document.lineAt(position);
            const text = line.text.substring(0, position.character);
            if (text.endsWith('du.')) {
                const items = [
                    new vscode.CompletionItem('generate', vscode.CompletionItemKind.Function),
                    new vscode.CompletionItem('search', vscode.CompletionItemKind.Function),
                    new vscode.CompletionItem('explain', vscode.CompletionItemKind.Function),
                ];
                return items;
            }
            return [];
        }
    };
    context.subscriptions.push(vscode.languages.registerCompletionItemProvider('*', provider, '.'));
}
function registerHover(context) {
    const provider = {
        provideHover: async (document, position) => {
            const word = document.getText(document.getWordRangeAtPosition(position));
            return new vscode.Hover(`DevUtils: ${word}`);
        }
    };
    context.subscriptions.push(vscode.languages.registerHoverProvider('*', provider));
}
async function runDevUtilsCommand(args) {
    try {
        const exePath = findDevUtils();
        if (!exePath) {
            vscode.window.showErrorMessage('DevUtils not found. Install from https://devutils.ai');
            return;
        }
        const result = await new Promise((resolve, reject) => {
            const proc = (0, child_process_1.spawn)(exePath, args);
            let output = '';
            proc.stdout.on('data', (data) => output += data);
            proc.on('close', (code) => {
                if (code === 0)
                    resolve(output);
                else
                    reject(new Error(`Exit code: ${code}`));
            });
            proc.on('error', reject);
        });
        vscode.window.showInformationMessage(result.substring(0, 100));
    }
    catch (e) {
        vscode.window.showErrorMessage(`Error: ${e}`);
    }
}
function findDevUtils() {
    const locations = [
        path.join(process.cwd(), 'devutils.exe'),
        path.join(__dirname, '../../target/release/devutils.exe'),
        path.join(__dirname, '../../target/debug/devutils.exe'),
        'C:\\Users\\arvin\\Documents\\OPENCODE\\devutils_rs\\target\\release\\devutils.exe',
    ];
    const { existsSync } = require('fs');
    for (const loc of locations) {
        if (existsSync(loc))
            return loc;
    }
    return 'devutils';
}
