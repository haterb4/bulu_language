import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('Bulu Language Server');
    outputChannel.appendLine('Bulu extension activating...');

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('bulu.restartLanguageServer', async () => {
            await restartLanguageServer();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('bulu.showOutputChannel', () => {
            outputChannel.show();
        })
    );

    // Start the language server
    startLanguageServer(context);

    outputChannel.appendLine('Bulu extension activated');
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

async function startLanguageServer(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('bulu');
    
    if (!config.get<boolean>('lsp.enabled', true)) {
        outputChannel.appendLine('Language server is disabled in settings');
        return;
    }

    const lspPath = config.get<string>('lsp.path', 'bulu_lsp');
    
    // Check if LSP server exists
    try {
        const { execSync } = require('child_process');
        execSync(`${lspPath} --version`, { stdio: 'ignore' });
    } catch (error) {
        const message = `Bulu Language Server not found at: ${lspPath}. Please install it or configure the path in settings.`;
        outputChannel.appendLine(`ERROR: ${message}`);
        
        const action = await vscode.window.showErrorMessage(
            message,
            'Open Settings',
            'Dismiss'
        );
        
        if (action === 'Open Settings') {
            vscode.commands.executeCommand('workbench.action.openSettings', 'bulu.lsp.path');
        }
        return;
    }

    const serverOptions: ServerOptions = {
        command: lspPath,
        args: [],
        transport: TransportKind.stdio
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'bulu' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.bu')
        },
        outputChannel: outputChannel,
        traceOutputChannel: outputChannel
    };

    client = new LanguageClient(
        'buluLanguageServer',
        'Bulu Language Server',
        serverOptions,
        clientOptions
    );

    try {
        await client.start();
        outputChannel.appendLine('Language server started successfully');
        
        vscode.window.showInformationMessage('Bulu Language Server is running');
    } catch (error) {
        outputChannel.appendLine(`Failed to start language server: ${error}`);
        vscode.window.showErrorMessage(`Failed to start Bulu Language Server: ${error}`);
    }
}

async function restartLanguageServer() {
    outputChannel.appendLine('Restarting language server...');
    
    if (client) {
        await client.stop();
        client = undefined;
    }
    
    const context = (global as any).extensionContext;
    if (context) {
        await startLanguageServer(context);
    }
    
    vscode.window.showInformationMessage('Bulu Language Server restarted');
}
