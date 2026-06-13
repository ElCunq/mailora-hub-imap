import re

file_path = "/home/cunq/Desktop/mailora-hub-imap/static/js/components/message-preview.js"
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

replacement = """    } catch (err) {
        if (err.name === 'AbortError') {
            console.log('Fetch aborted for older email click');
            return;
        }
        c.innerHTML = `<div class="empty-state"><div style="color:var(--accent-red)">Hata: ${err.message}</div></div>`;
        return;
    }"""
    
content = content.replace("""    } catch (err) {
        c.innerHTML = `<div class="empty-state"><div style="color:var(--accent-red)">Hata: ${err.message}</div></div>`;
        return;
    }""", replacement)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
print("AbortError handling fixed!")
