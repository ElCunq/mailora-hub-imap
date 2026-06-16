import json

with open('/home/cunq/.gemini/antigravity/brain/89a3eab7-9ce4-41b8-bf81-699b84bf11c6/.system_generated/logs/overview.txt', 'r', encoding='utf-8') as f:
    for line in f:
        try:
            data = json.loads(line)
            if 'tool_calls' in data:
                for call in data['tool_calls']:
                    if call['name'] == 'multi_replace_file_content':
                        args = call['args']
                        if 'admin.html' in args.get('TargetFile', ''):
                            chunks = args.get('ReplacementChunks', '')
                            if 'Güvenlik' in chunks or 'security' in chunks:
                                print(f"Found in step {data['step_index']}:")
                                print(chunks)
        except:
            pass
