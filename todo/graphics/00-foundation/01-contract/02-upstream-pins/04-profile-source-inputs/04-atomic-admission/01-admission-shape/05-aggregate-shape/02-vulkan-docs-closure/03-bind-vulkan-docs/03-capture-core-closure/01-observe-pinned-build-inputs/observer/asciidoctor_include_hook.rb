require "json"

module WebboxDocsIncludeTrace
  def self.log(file, path, lineno)
    target = ENV.fetch("WEBBOXVM_INCLUDE_TRACE")
    File.open(target, "a") { |stream| stream.puts(JSON.generate({"file" => file, "path" => path, "line" => lineno})) }
  end
end

require "asciidoctor"
require "asciidoctor/reader"

module WebboxDocsIncludeHook
  def push_include(data, file = nil, path = nil, lineno = 1, attributes = {})
    result = super
    WebboxDocsIncludeTrace.log(file, path, lineno)
    result
  end
end

Asciidoctor::PreprocessorReader.prepend(WebboxDocsIncludeHook)
