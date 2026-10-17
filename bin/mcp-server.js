#!/usr/bin/env node

/**
 * context-grill MCP (Model Context Protocol) Stdio Server
 * Exposes cgrill tools to Cursor, Windsurf, Claude Desktop, and AGY.
 * Zero-dependency JSON-RPC 2.0 stdio implementation.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const { execFileSync } = require('child_process');
const readline = require('readline');

const SERVER_NAME = 'context-grill-mcp';
const SERVER_VERSION = '0.1.0';
const PROTOCOL_VERSION = '2024-11-05';

function findBinary() {
  const repoRoot = path.resolve(__dirname, '..');
  const binName = process.platform === 'win32' ? 'cgrill.exe' : 'cgrill';

  const candidates = [
    path.join(repoRoot, 'target', 'release', binName),
    path.join(repoRoot, 'target', 'debug', binName),
    path.join(os.homedir(), '.cgrill', 'bin', binName),
    path.join(os.homedir(), '.cargo', 'bin', binName),
    '/usr/local/bin/' + binName,
    '/opt/homebrew/bin/' + binName,
  ];

  for (const c of candidates) {
    if (fs.existsSync(c)) {
      return c;
    }
  }

  // Fallback to searching PATH
  const pathDirs = (process.env.PATH || '').split(path.delimiter);
  for (const d of pathDirs) {
    const p = path.join(d, binName);
    if (fs.existsSync(p)) return p;
  }

  return 'cgrill';
}

const CGRILL_BIN = findBinary();

function runCgrill(args, cwd = process.cwd()) {
  try {
    const stdout = execFileSync(CGRILL_BIN, args, {
      cwd,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
      env: process.env,
    });
    return { success: true, stdout };
  } catch (err) {
    return {
      success: false,
      stdout: err.stdout || '',
      stderr: err.stderr || err.message,
      status: err.status || 1,
    };
  }
}

const TOOLS = [
  {
    name: 'cgrill_scan',
    description: 'Scan a task prompt for ambiguity, vague terms, and generate Socratic questions before taking action.',
    inputSchema: {
      type: 'object',
      properties: {
        prompt: {
          type: 'string',
          description: 'The user prompt or task description to analyze',
        },
      },
      required: ['prompt'],
    },
  },
  {
    name: 'cgrill_env',
    description: 'Inspect local runtime reality (listening ports, running daemons, dirty git tree, stale build caches).',
    inputSchema: {
      type: 'object',
      properties: {
        dir: {
          type: 'string',
          description: 'Directory to inspect (defaults to current working directory)',
        },
      },
    },
  },
  {
    name: 'cgrill_patch_guard',
    description: 'Verify a code patch before modifying disk. Prevents line drift, off-by-one errors, and unbalanced brackets/syntax clipping.',
    inputSchema: {
      type: 'object',
      properties: {
        file: {
          type: 'string',
          description: 'Absolute or relative path to the target file',
        },
        start: {
          type: 'integer',
          description: 'Starting line number (1-indexed)',
        },
        end: {
          type: 'integer',
          description: 'Ending line number (1-indexed)',
        },
        target: {
          type: 'string',
          description: 'Exact target text to replace in the file',
        },
        replacement: {
          type: 'string',
          description: 'New replacement text to insert',
        },
      },
      required: ['file', 'start', 'end', 'target', 'replacement'],
    },
  },
  {
    name: 'cgrill_contract_init',
    description: 'Initialize a new Ground Truth Contract (.cgrill/contract.json) to lock goal, diff appetite, allowed files, and acceptance criteria.',
    inputSchema: {
      type: 'object',
      properties: {
        goal: {
          type: 'string',
          description: 'The exact goal or task statement',
        },
        diff: {
          type: 'string',
          description: 'Diff appetite (e.g. "surgical (<20 lines)", "targeted (<100 lines)")',
        },
        allowed: {
          type: 'string',
          description: 'Comma-separated allowed file paths that may be mutated',
        },
        forbidden: {
          type: 'string',
          description: 'Comma-separated forbidden file paths that must remain untouched',
        },
        criteria: {
          type: 'string',
          description: 'Comma-separated acceptance test commands that prove completion',
        },
      },
      required: ['goal'],
    },
  },
  {
    name: 'cgrill_contract_lock',
    description: 'Lock the current contract to prevent scope creep and emit the compact 6-line memory pin.',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'cgrill_contract_pin',
    description: 'Retrieve the 6-line pinned contract memory block to maintain invariants across multi-turn LLM loops.',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'cgrill_contract_check_write',
    description: 'Check if mutating a specific file path is permitted under the locked contract.',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'File path to verify against allowed and forbidden contract scopes',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'cgrill_check',
    description: 'Run full pre-flight verification: inspect runtime environment and verify active contract invariants.',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
];

function handleToolCall(name, args, cwd) {
  switch (name) {
    case 'cgrill_scan': {
      const res = runCgrill(['scan', args.prompt, '--json'], cwd);
      return res.stdout || res.stderr;
    }
    case 'cgrill_env': {
      const cmdArgs = ['env', '--json'];
      if (args.dir) cmdArgs.push('--dir', args.dir);
      const res = runCgrill(cmdArgs, cwd);
      return res.stdout || res.stderr;
    }
    case 'cgrill_patch_guard': {
      const cmdArgs = [
        'patch-guard',
        '--file',
        args.file,
        '--start',
        String(args.start),
        '--end',
        String(args.end),
        '--target',
        args.target,
        '--replacement',
        args.replacement,
        '--json',
      ];
      const res = runCgrill(cmdArgs, cwd);
      return res.stdout || res.stderr;
    }
    case 'cgrill_contract_init': {
      const cmdArgs = ['contract', 'init', '--goal', args.goal];
      if (args.diff) cmdArgs.push('--diff', args.diff);
      if (args.allowed) cmdArgs.push('--allowed', args.allowed);
      if (args.forbidden) cmdArgs.push('--forbidden', args.forbidden);
      if (args.criteria) cmdArgs.push('--criteria', args.criteria);
      const res = runCgrill(cmdArgs, cwd);
      return res.stdout || res.stderr;
    }
    case 'cgrill_contract_lock': {
      const res = runCgrill(['contract', 'lock'], cwd);
      return res.stdout || res.stderr;
    }
    case 'cgrill_contract_pin': {
      const res = runCgrill(['contract', 'pin'], cwd);
      return res.stdout || res.stderr;
    }
    case 'cgrill_contract_check_write': {
      const res = runCgrill(['contract', 'check-write', args.path], cwd);
      return res.stdout || res.stderr;
    }
    case 'cgrill_check': {
      const res = runCgrill(['check'], cwd);
      return res.stdout || res.stderr;
    }
    default:
      throw new Error(`Unknown tool: ${name}`);
  }
}

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

rl.on('line', (line) => {
  if (!line.trim()) return;
  try {
    const req = JSON.parse(line);
    handleRequest(req);
  } catch (err) {
    sendResponse({
      jsonrpc: '2.0',
      id: null,
      error: { code: -32700, message: 'Parse error: ' + err.message },
    });
  }
});

function sendResponse(resp) {
  process.stdout.write(JSON.stringify(resp) + '\n');
}

function handleRequest(req) {
  const { id, method, params } = req;

  if (method === 'initialize') {
    sendResponse({
      jsonrpc: '2.0',
      id,
      result: {
        protocolVersion: PROTOCOL_VERSION,
        capabilities: {
          tools: {},
        },
        serverInfo: {
          name: SERVER_NAME,
          version: SERVER_VERSION,
        },
      },
    });
    return;
  }

  if (method === 'notifications/initialized') {
    // Client notification, no reply needed
    return;
  }

  if (method === 'ping') {
    sendResponse({ jsonrpc: '2.0', id, result: {} });
    return;
  }

  if (method === 'tools/list') {
    sendResponse({
      jsonrpc: '2.0',
      id,
      result: {
        tools: TOOLS,
      },
    });
    return;
  }

  if (method === 'tools/call') {
    const { name, arguments: args } = params || {};
    try {
      const outputText = handleToolCall(name, args || {}, process.cwd());
      sendResponse({
        jsonrpc: '2.0',
        id,
        result: {
          content: [
            {
              type: 'text',
              text: outputText,
            },
          ],
        },
      });
    } catch (err) {
      sendResponse({
        jsonrpc: '2.0',
        id,
        result: {
          isError: true,
          content: [
            {
              type: 'text',
              text: `Error executing ${name}: ${err.message}`,
            },
          ],
        },
      });
    }
    return;
  }

  // Unknown method
  if (id !== undefined && id !== null) {
    sendResponse({
      jsonrpc: '2.0',
      id,
      error: {
        code: -32601,
        message: `Method not found: ${method}`,
      },
    });
  }
}
