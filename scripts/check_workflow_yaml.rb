#!/usr/bin/env ruby
# frozen_string_literal: true

# Checks GitHub workflow and composite action files: each parses as YAML, has no duplicate mapping
# keys (which YAML loaders accept silently and GitHub rejects as a "workflow file issue"; AgentX's
# check_workflow_yaml.rb) and has the shape GitHub expects. Prints `workflow yaml ok` once every given
# file passes; exits 1 with the problems otherwise.
#
#   ruby scripts/check_workflow_yaml.rb .github/workflows/*.yml .github/actions/*/action.yml

require "yaml"

def check_steps(steps, where, problems)
  unless steps.is_a?(Array) && !steps.empty?
    problems << "#{where}: `steps` must be a non-empty list"
    return
  end
  steps.each_with_index do |step, index|
    unless step.is_a?(Hash)
      problems << "#{where}: step #{index + 1} is not a mapping"
      next
    end
    if step.key?("uses") == step.key?("run")
      problems << "#{where}: step #{index + 1} needs exactly one of `uses` and `run`"
    end
    if step.key?("uses") && step["uses"].to_s !~ /\A[\w.\-\/]+@[\w.\-]+\z|\A\.\/[\w.\-\/]+\z|\Adocker:\/\//
      problems << "#{where}: step #{index + 1} `uses: #{step['uses']}` names no version (owner/repo@ref)"
    end
  end
end

# Duplicate keys at any depth, from the parse tree (the loaded document keeps only the last one).
def check_duplicates(node, file, path, problems)
  return unless node.respond_to?(:children) && node.children
  if node.is_a?(Psych::Nodes::Mapping)
    pairs = node.children.each_slice(2).to_a
    keys = pairs.map { |key, _| key.respond_to?(:value) ? key.value : key.to_s }
    keys.group_by(&:itself).select { |_, all| all.size > 1 }.each_key do |key|
      problems << "#{file}: duplicate key `#{key}` at #{path.empty? ? '/' : path}"
    end
    pairs.each_with_index { |(_, value), i| check_duplicates(value, file, "#{path}/#{keys[i]}", problems) }
  else
    node.children.each { |child| check_duplicates(child, file, path, problems) }
  end
end

def check_workflow(doc, path, problems)
  # YAML 1.1 reads the `on:` key as `true`.
  triggers = doc.key?("on") ? doc["on"] : doc[true]
  problems << "#{path}: no `name`" unless doc["name"].is_a?(String) && !doc["name"].empty?
  problems << "#{path}: no `on` triggers" if triggers.nil?
  jobs = doc["jobs"]
  unless jobs.is_a?(Hash) && !jobs.empty?
    problems << "#{path}: `jobs` must be a non-empty mapping"
    return
  end
  jobs.each do |id, job|
    where = "#{path}: job `#{id}`"
    unless job.is_a?(Hash)
      problems << "#{where} is not a mapping"
      next
    end
    if job.key?("uses")
      next # a reusable workflow call
    end
    problems << "#{where} has no `runs-on`" unless job.key?("runs-on")
    check_steps(job["steps"], where, problems)
    Array(job["needs"]).each do |need|
      problems << "#{where} needs unknown job `#{need}`" unless jobs.key?(need)
    end
  end
end

def check_action(doc, path, problems)
  problems << "#{path}: no `name`" unless doc["name"].is_a?(String) && !doc["name"].empty?
  runs = doc["runs"]
  unless runs.is_a?(Hash) && runs["using"]
    problems << "#{path}: `runs.using` is missing"
    return
  end
  check_steps(runs["steps"], "#{path}: runs", problems) if runs["using"] == "composite"
end

files = ARGV
if files.empty?
  warn "usage: ruby scripts/check_workflow_yaml.rb <workflow or action.yml>..."
  exit 2
end

problems = []
files.each do |path|
  begin
    check_duplicates(Psych.parse_file(path), path, "", problems)
    doc = YAML.safe_load(File.read(path), aliases: false)
  rescue Psych::SyntaxError, Errno::ENOENT => e
    problems << "#{path}: #{e.message}"
    next
  end
  unless doc.is_a?(Hash)
    problems << "#{path}: not a YAML mapping"
    next
  end
  if File.basename(path).start_with?("action.")
    check_action(doc, path, problems)
  else
    check_workflow(doc, path, problems)
  end
end

if problems.empty?
  puts "workflow yaml ok"
else
  problems.each { |problem| warn problem }
  exit 1
end
