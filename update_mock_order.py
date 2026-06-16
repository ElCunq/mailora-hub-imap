import re
import json

file_path = 'static/Example/js/mock-app.js'
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# We need to extract the mockData array string
match = re.search(r'const mockData = \[(.*?)\];\n\nlet state =', content, re.DOTALL)
if match:
    array_content = match.group(1)
    # Split the array content by "    },"
    items = array_content.split('    },')
    
    # Clean up and re-add the suffix
    clean_items = []
    for idx, item in enumerate(items):
        item = item.strip()
        if not item:
            continue
        if not item.endswith('}'):
            item = item + '\n    }'
        clean_items.append(item)
    
    # We want:
    # 0: id 12 (Global Tech) - Top
    # id: 13, 14, 15, 16 mixed inside.
    
    # Let's identify the items
    id_12 = [x for x in clean_items if 'id: 12,' in x][0]
    id_13 = [x for x in clean_items if 'id: 13,' in x][0]
    id_14 = [x for x in clean_items if 'id: 14,' in x][0]
    id_15 = [x for x in clean_items if 'id: 15,' in x][0]
    id_16 = [x for x in clean_items if 'id: 16,' in x][0]
    
    id_1 = [x for x in clean_items if 'id: 1,' in x][0]
    id_2 = [x for x in clean_items if 'id: 2,' in x][0]
    id_3 = [x for x in clean_items if 'id: 3,' in x][0]
    id_4 = [x for x in clean_items if 'id: 4,' in x][0]
    id_5 = [x for x in clean_items if 'id: 5,' in x][0]
    id_6 = [x for x in clean_items if 'id: 6,' in x][0]
    id_7 = [x for x in clean_items if 'id: 7,' in x][0]
    id_8 = [x for x in clean_items if 'id: 8,' in x][0]
    id_9 = [x for x in clean_items if 'id: 9,' in x][0]
    id_10 = [x for x in clean_items if 'id: 10,' in x][0]
    id_11 = [x for x in clean_items if 'id: 11,' in x][0]

    # Reorder
    new_order = [
        id_12,
        id_1,
        id_14,
        id_2,
        id_15,
        id_3,
        id_7,
        id_13,
        id_4,
        id_6,
        id_16,
        id_8,
        id_5,
        id_9,
        id_10,
        id_11
    ]

    new_array_content = ',\n    '.join(new_order)
    
    new_content = content[:match.start()] + 'const mockData = [\n    ' + new_array_content + '\n];\n\nlet state =' + content[match.end():]
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Reordered successfully!")
else:
    print("mockData array not found!")
